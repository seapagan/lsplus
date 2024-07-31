use nix::unistd::{Group, User};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

use crate::utils::format;
use crate::Params;
use std::path::Path;

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
