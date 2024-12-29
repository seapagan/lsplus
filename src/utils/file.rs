use nix::unistd::{Group, User};
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::SystemTime;

use inline_colorization::*;

use crate::structs::FileInfo;
use crate::utils;
use crate::utils::format;
use crate::Params;

pub fn get_file_details(
    metadata: &fs::Metadata,
) -> (String, String, u64, u64, SystemTime, String, String, bool) {
    let file_type = if metadata.is_dir() {
        "d"
    } else if metadata.is_file() {
        "-"
    } else if metadata.is_symlink() {
        "l"
    } else {
        "?"
    }
    .to_string();

    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let rwx_mode = format::mode_to_rwx(mode);

    let nlink = metadata.nlink();
    let size = metadata.size();

    let user = get_username(metadata.uid());
    let group = get_groupname(metadata.gid());

    let mtime = metadata.modified().unwrap();

    #[cfg(unix)]
    let executable = metadata.permissions().mode() & 0o111 != 0;

    // for now just return false under windows
    #[cfg(windows)]
    let executable = false;

    (
        file_type, rwx_mode, nlink, size, mtime, user, group, executable,
    )
}

pub fn collect_file_names(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let path_metadata = fs::symlink_metadata(path)?;

    if !path_metadata.is_dir() {
        // If it's a file or symlink, add it directly to the file_names vector
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        file_names.push(file_name);
    } else {
        // If it's a directory, read its entries
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if params.show_all || params.almost_all {
                    true
                } else {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.'))
                        .unwrap_or(false)
                }
            })
            .collect();

        // Sort entries alphabetically, ignoring leading dots
        entries.sort_by(|a, b| {
            let a_name = a
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            let b_name = b
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            a_name.cmp(&b_name)
        });

        // Separate directories and files if dirs_first is true
        if params.dirs_first {
            let (dirs, files): (Vec<_>, Vec<_>) =
                entries.into_iter().partition(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                });

            entries = dirs.into_iter().chain(files).collect();
        }

        if !params.almost_all && params.show_all {
            file_names.push(".".to_string());
            file_names.push("..".to_string());
        }

        for entry in entries {
            file_names.push(entry.file_name().to_string_lossy().into_owned())
        }
    }
    Ok(file_names)
}

pub fn get_username(uid: u32) -> String {
    match User::from_uid(uid.into()) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

pub fn get_groupname(gid: u32) -> String {
    match Group::from_gid(gid.into()) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}

pub fn collect_file_info(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<FileInfo>> {
    let mut file_info = Vec::new();
    let metadata = fs::symlink_metadata(path)?;

    if metadata.is_dir() {
        let file_names = utils::file::collect_file_names(path, params)?;

        for file_name in file_names {
            let full_path = path.join(&file_name);
            if let Ok(info) = create_file_info(&full_path, params) {
                file_info.push(info);
            }
        }
    } else if let Ok(info) = create_file_info(path, params) {
        file_info.push(info);
    }
    Ok(file_info)
}

pub fn create_file_info(path: &Path, params: &Params) -> io::Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    let item_icon = if params.no_icons {
        None
    } else {
        Some(utils::icons::get_item_icon(
            &metadata,
            &path.to_string_lossy(),
        ))
    };
    let (file_type, mode, nlink, size, mtime, user, group, executable) =
        utils::file::get_file_details(&metadata);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    if params.append_slash && metadata.is_dir() {
        file_name.push('/');
    }

    let display_name = if metadata.is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                let target_path = if target.is_relative() {
                    path.parent().unwrap_or(Path::new("")).join(target)
                } else {
                    target
                };
                if params.long_format {
                    if target_path.exists() {
                        format!(
                            "{color_cyan}{} -> {}",
                            file_name,
                            target_path.display()
                        )
                    } else {
                        format!(
                            "{color_cyan}{} -> {} {color_red}[Broken Link]",
                            file_name,
                            target_path.display()
                        )
                    }
                } else {
                    format!("{color_cyan}{}", file_name)
                }
            }
            Err(_) => {
                if params.long_format {
                    format!("{color_red}{} -> (unreadable)", file_name)
                } else {
                    format!("{color_cyan}{}", file_name)
                }
            }
        }
    } else if metadata.is_dir() {
        format!("{color_blue}{}", file_name)
    } else if executable {
        format!("{style_bold}{color_green}{}", file_name)
    } else {
        file_name.clone()
    };

    Ok(FileInfo {
        file_type,
        mode,
        nlink,
        user,
        group,
        size,
        mtime,
        item_icon,
        display_name,
        full_path: path.to_path_buf(),
    })
}

pub fn check_display_name(info: &FileInfo) -> String {
    match &info.full_path.to_string_lossy() {
        p if p.ends_with("/.") => format!("{color_blue}."),
        p if p.ends_with("/..") => format!("{color_blue}.."),
        _ => info.display_name.to_string(),
    }
}
