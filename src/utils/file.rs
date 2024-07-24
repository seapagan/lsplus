use nix::unistd::{Group, User};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

use crate::utils::format;
use crate::{Params, PathBuf};

pub fn get_file_name_with_slash(
    metadata: &fs::Metadata,
    file_name: &str,
    append_slash: bool,
) -> String {
    if metadata.is_dir() && append_slash {
        format!("{}/", file_name)
    } else {
        file_name.to_string()
    }
}

pub fn get_file_details(
    metadata: &fs::Metadata,
) -> (String, String, u64, u64, SystemTime, String, String) {
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

    (file_type, rwx_mode, nlink, size, mtime, user, group)
}

pub fn calculate_max_name_length(file_names: &[String]) -> usize {
    file_names.iter().map(|name| name.len()).max().unwrap_or(0) + 2 // Adding space between columns
}

pub fn collect_file_names(
    path: &String,
    params: &Params,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let path_metadata = fs::symlink_metadata(path)?;

    if !path_metadata.is_dir() {
        // If it's a file or symlink, add it directly to the file_names vector
        let file_name = PathBuf::from(path)
            // .file_name()
            // .unwrap()
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
                .into_string()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            let b_name = b
                .file_name()
                .into_string()
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
            if params.append_slash {
                file_names = vec!["./".to_string(), "../".to_string()];
            } else {
                file_names = vec![".".to_string(), "..".to_string()];
            }
        }

        for entry in entries {
            let metadata = fs::symlink_metadata(entry.path())?;
            let mut file_name = entry.file_name().into_string().unwrap();
            file_name = get_file_name_with_slash(
                &metadata,
                &file_name,
                params.append_slash,
            );
            file_names.push(file_name);
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
