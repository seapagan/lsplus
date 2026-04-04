use colored_text::{Colorize, StyledText};
use nix::unistd::{Group, User};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::Params;
use crate::structs::FileInfo;
use crate::structs::NameStyle;
use crate::utils;
use crate::utils::format;
use crate::utils::gitignore::GitignoreCache;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

struct FileDetails {
    file_type: String,
    mode: String,
    nlink: u64,
    size: u64,
    mtime: SystemTime,
    user: String,
    group: String,
}

pub(crate) struct DirectoryEntryData {
    pub file_name: OsString,
    pub path: PathBuf,
    pub is_dir: Result<bool, io::Error>,
}

fn get_file_details(metadata: &fs::Metadata) -> FileDetails {
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

    FileDetails {
        file_type,
        mode: rwx_mode,
        nlink,
        size,
        mtime,
        user,
        group,
    }
}

pub fn collect_file_names(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    // First get the symlink metadata to check if this is a symlink
    let symlink_metadata = fs::symlink_metadata(path)?;

    // If it's a symlink, get the actual metadata by following it
    let path_metadata = if symlink_metadata.is_symlink() {
        fs::metadata(path)?
    } else {
        symlink_metadata
    };

    if !path_metadata.is_dir() {
        // If it's a file or symlink, add it directly to the file_names vector
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        file_names.push(file_name);
    } else {
        let entries = fs::read_dir(path)?
            .map(|entry_result| {
                entry_result.map(|entry| DirectoryEntryData {
                    file_name: entry.file_name(),
                    path: entry.path(),
                    is_dir: entry
                        .file_type()
                        .map(|file_type| file_type.is_dir()),
                })
            })
            .collect();

        file_names.extend(collect_visible_file_names(path, entries, params));
    }
    Ok(file_names)
}

pub(crate) fn collect_visible_file_names(
    path: &Path,
    entries: Vec<Result<DirectoryEntryData, io::Error>>,
    params: &Params,
) -> Vec<String> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut visible_entries = Vec::new();

    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                if params.show_all
                    || params.almost_all
                    || !entry_name_is_hidden(&entry.file_name)
                {
                    visible_entries.push(entry);
                }
            }
            Err(err) => report_path_error(path, &err),
        }
    }

    visible_entries.sort_by_cached_key(|entry| sort_key(&entry.file_name));

    if params.dirs_first {
        let (dirs, files): (Vec<_>, Vec<_>) = visible_entries
            .into_iter()
            .partition(|entry| match &entry.is_dir {
                Ok(is_dir) => *is_dir,
                Err(err) => {
                    report_path_error(&entry.path, err);
                    false
                }
            });

        visible_entries = dirs.into_iter().chain(files).collect();
    }

    let mut file_names = Vec::new();
    if !params.almost_all && params.show_all {
        file_names.push(".".to_string());
        file_names.push("..".to_string());
    }

    for entry in visible_entries {
        file_names.push(entry.file_name.to_string_lossy().into_owned())
    }

    file_names
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
    let mut gitignore_cache = GitignoreCache::default();
    let symlink_metadata = fs::symlink_metadata(path)?;

    // If it's a symlink, try following it to determine whether it points to a
    // directory. Broken symlinks should still be displayed as entries.
    let is_dir = if symlink_metadata.is_symlink() {
        fs::metadata(path)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false)
    } else {
        symlink_metadata.is_dir()
    };

    if is_dir {
        let file_names = utils::file::collect_file_names(path, params)?;
        append_file_info_for_names(
            &mut file_info,
            path,
            &file_names,
            params,
            &mut gitignore_cache,
        );
    } else {
        let info = create_file_info_from_metadata_with_gitignore(
            path,
            &symlink_metadata,
            params,
            &mut gitignore_cache,
        );
        file_info.push(info);
    }
    Ok(file_info)
}

pub(crate) fn append_file_info_for_names(
    file_info: &mut Vec<FileInfo>,
    path: &Path,
    file_names: &[String],
    params: &Params,
    gitignore_cache: &mut GitignoreCache,
) {
    if file_names.is_empty() {
        return;
    }

    for file_name in file_names {
        let full_path = path.join(file_name);
        match create_file_info_with_gitignore(
            &full_path,
            params,
            gitignore_cache,
        ) {
            Ok(info) => file_info.push(info),
            Err(err) => report_path_error(&full_path, &err),
        }
    }
}

pub fn create_file_info(path: &Path, params: &Params) -> io::Result<FileInfo> {
    let mut gitignore_cache = GitignoreCache::default();
    create_file_info_with_gitignore(path, params, &mut gitignore_cache)
}

fn create_file_info_with_gitignore(
    path: &Path,
    params: &Params,
    gitignore_cache: &mut GitignoreCache,
) -> io::Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(create_file_info_from_metadata_with_gitignore(
        path,
        &metadata,
        params,
        gitignore_cache,
    ))
}

fn create_file_info_from_metadata_with_gitignore(
    path: &Path,
    metadata: &fs::Metadata,
    params: &Params,
    gitignore_cache: &mut GitignoreCache,
) -> FileInfo {
    let item_icon = if params.no_icons {
        None
    } else {
        Some(utils::icons::get_item_icon(metadata, path))
    };
    let details = utils::file::get_file_details(metadata);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    let mut safe_file_name = sanitize_for_terminal(&file_name);

    if params.append_slash && metadata.is_dir() {
        safe_file_name.push('/');
    }

    let ignored = params.gitignore
        && gitignore_cache.is_ignored(path, metadata.is_dir());

    let (display_name, short_name, name_style) = if metadata.is_symlink() {
        (
            if ignored {
                format_symlink_display_name_with_dim(
                    &safe_file_name,
                    path,
                    fs::read_link(path),
                    params,
                    true,
                )
            } else {
                format_symlink_display_name(
                    &safe_file_name,
                    path,
                    fs::read_link(path),
                    params,
                )
            },
            format!("{safe_file_name}{}", symlink_short_suffix(params)),
            NameStyle::Symlink,
        )
    } else {
        (
            colorize_name_by_metadata(&safe_file_name, metadata, ignored),
            safe_file_name.clone(),
            name_style_by_metadata(metadata),
        )
    };

    FileInfo {
        file_type: details.file_type,
        mode: details.mode,
        nlink: details.nlink,
        user: details.user,
        group: details.group,
        size: details.size,
        mtime: details.mtime,
        item_icon,
        short_name,
        display_name,
        name_style,
        dimmed: ignored,
        full_path: path.to_path_buf(),
    }
}

pub fn check_display_name(info: &FileInfo) -> String {
    match &info.full_path.to_string_lossy() {
        p if p.ends_with("/.") => ".".blue().to_string(),
        p if p.ends_with("/..") => "..".blue().to_string(),
        _ => info.display_name.to_string(),
    }
}

fn entry_name_is_hidden(name: &OsStr) -> bool {
    #[cfg(unix)]
    {
        name.as_bytes().starts_with(b".")
    }

    #[cfg(not(unix))]
    {
        name.to_string_lossy().starts_with('.')
    }
}

fn sort_key(name: &OsStr) -> Vec<u8> {
    #[cfg(unix)]
    {
        let bytes = name.as_bytes();
        let trimmed = bytes
            .iter()
            .skip_while(|byte| **byte == b'.')
            .copied()
            .collect::<Vec<_>>();
        trimmed
            .into_iter()
            .map(|byte| byte.to_ascii_lowercase())
            .collect()
    }

    #[cfg(not(unix))]
    {
        name.to_string_lossy()
            .trim_start_matches('.')
            .to_lowercase()
            .into_bytes()
    }
}

pub(crate) fn sanitize_for_terminal(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut sanitized = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\n' => sanitized.push_str("\\n"),
            '\r' => sanitized.push_str("\\r"),
            '\t' => sanitized.push_str("\\t"),
            _ if character.is_control() => {
                let code = character as u32;
                sanitized.push_str(&format!("\\x{code:02x}"));
            }
            _ => sanitized.push(character),
        }
    }
    sanitized
}

fn sanitize_path_for_terminal(path: &Path) -> String {
    sanitize_for_terminal(&path.to_string_lossy())
}

fn symlink_short_suffix(params: &Params) -> &'static str {
    if params.append_slash { "*" } else { "" }
}

fn apply_dim(style: StyledText, dimmed: bool) -> StyledText {
    if dimmed { style.dim() } else { style }
}

fn plain_text(text: impl Into<String>, dimmed: bool) -> String {
    apply_dim(StyledText::plain(text), dimmed).to_string()
}

fn name_style_by_metadata(metadata: &fs::Metadata) -> NameStyle {
    if metadata.is_symlink() {
        NameStyle::Symlink
    } else if metadata.is_dir() {
        NameStyle::Directory
    } else {
        #[cfg(unix)]
        let executable = metadata.permissions().mode() & 0o111 != 0;

        #[cfg(windows)]
        let executable = false;

        if executable {
            NameStyle::Executable
        } else {
            NameStyle::Plain
        }
    }
}

fn colorize_name_by_metadata(
    safe_name: &str,
    metadata: &fs::Metadata,
    dimmed: bool,
) -> String {
    if metadata.is_symlink() {
        apply_dim(safe_name.cyan(), dimmed).to_string()
    } else if metadata.is_dir() {
        apply_dim(safe_name.blue(), dimmed).to_string()
    } else {
        #[cfg(unix)]
        let executable = metadata.permissions().mode() & 0o111 != 0;

        #[cfg(windows)]
        let executable = false;

        if executable {
            apply_dim(safe_name.green().bold(), dimmed).to_string()
        } else {
            plain_text(safe_name, dimmed)
        }
    }
}

pub(crate) fn format_symlink_display_name(
    safe_file_name: &str,
    path: &Path,
    target: io::Result<PathBuf>,
    params: &Params,
) -> String {
    format_symlink_display_name_with_dim(
        safe_file_name,
        path,
        target,
        params,
        false,
    )
}

fn format_symlink_display_name_with_dim(
    safe_file_name: &str,
    path: &Path,
    target: io::Result<PathBuf>,
    params: &Params,
    dimmed: bool,
) -> String {
    match target {
        Ok(target) => {
            let target_path = if target.is_relative() {
                path.parent().unwrap_or(Path::new("")).join(target)
            } else {
                target
            };
            let display_target = sanitize_path_for_terminal(&target_path);
            if params.long_format {
                let display_target = fs::symlink_metadata(&target_path)
                    .map(|metadata| {
                        colorize_name_by_metadata(
                            &display_target,
                            &metadata,
                            dimmed,
                        )
                    })
                    .unwrap_or_else(|_| plain_text(&display_target, dimmed));

                if target_path.exists() {
                    format!(
                        "{}{}{}",
                        apply_dim(safe_file_name.cyan(), dimmed),
                        plain_text(" -> ", dimmed),
                        display_target
                    )
                } else {
                    format!(
                        "{}{}{}{}{}",
                        apply_dim(safe_file_name.cyan(), dimmed),
                        plain_text(" -> ", dimmed),
                        display_target,
                        plain_text(" ", dimmed),
                        apply_dim("[Broken Link]".red(), dimmed)
                    )
                }
            } else {
                apply_dim(
                    format!(
                        "{safe_file_name}{}",
                        symlink_short_suffix(params)
                    )
                    .cyan(),
                    dimmed,
                )
                .to_string()
            }
        }
        Err(_) => {
            if params.long_format {
                apply_dim(
                    format!("{safe_file_name} -> (unreadable)").red(),
                    dimmed,
                )
                .to_string()
            } else {
                apply_dim(
                    format!(
                        "{safe_file_name}{}",
                        symlink_short_suffix(params)
                    )
                    .cyan(),
                    dimmed,
                )
                .to_string()
            }
        }
    }
}

pub(crate) fn format_path_error(path: &Path, err: &io::Error) -> String {
    format!("lsplus: {}: {}", sanitize_path_for_terminal(path), err)
}

fn report_path_error(path: &Path, err: &io::Error) {
    eprintln!("{}", format_path_error(path, err));
}
