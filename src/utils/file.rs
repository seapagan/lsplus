//! Filesystem inspection and entry-name formatting.
//!
//! This module collects the metadata needed by renderers, applies visibility
//! and directory-ordering rules, sanitizes terminal output, and prepares
//! styled names for regular files, directories, symlinks, and gitignored
//! entries.

use colored_text::{Colorize, StyledText};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::IndicatorStyle;
use crate::Params;
use crate::platform::{self, LongFormatFileType};
use crate::structs::FileInfo;
use crate::structs::NameStyle;
use crate::utils::{self, gitignore::GitignoreCache};

/// Directory entry data captured before visibility filtering and sorting.
pub(crate) struct DirectoryEntryData {
    /// Raw entry name from `read_dir`.
    pub file_name: OsString,
    /// Full entry path.
    pub path: PathBuf,
    /// Directory classification captured while reading the entry.
    pub is_dir: Result<bool, io::Error>,
}

/// Look up a username, falling back to the numeric UID.
pub fn get_username(uid: u32) -> String {
    platform::get_username(uid)
}

/// Look up a group name, falling back to the numeric GID.
pub fn get_groupname(gid: u32) -> String {
    platform::get_groupname(gid)
}

/// Return displayable names for a file path or visible entries in a directory.
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

/// Return visible entry names for a directory after sorting and filtering.
///
/// Hidden-file handling follows the parsed params, and `dirs_first` preserves
/// the sorted order within the directory and non-directory groups.
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

/// Collect display metadata for a file or every visible entry in a directory.
///
/// Directory symlinks are followed for directory traversal decisions, while
/// broken symlinks remain listable as their own entries.
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

/// Append display metadata for a list of names under one directory.
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

/// Build display metadata for a single filesystem path.
pub fn create_file_info(path: &Path, params: &Params) -> io::Result<FileInfo> {
    let mut gitignore_cache = GitignoreCache::default();
    create_file_info_with_gitignore(path, params, &mut gitignore_cache)
}

pub(crate) fn create_file_info_with_gitignore(
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

pub(crate) fn create_file_info_from_metadata_with_gitignore(
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
    let details = platform::file_details(metadata);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    let safe_file_name = sanitize_for_terminal(&file_name);
    let indicated_file_name =
        format_name_with_indicator(&safe_file_name, metadata, params);

    let ignored = params.gitignore
        && gitignore_cache.is_ignored(path, metadata.is_dir());

    let (display_name, short_name, name_style) = if metadata.is_symlink() {
        (
            format_symlink_display_name_with_dim(
                &indicated_file_name,
                path,
                fs::read_link(path),
                params,
                ignored,
            ),
            indicated_file_name.clone(),
            NameStyle::Symlink,
        )
    } else {
        (
            colorize_name_by_metadata(&indicated_file_name, metadata, ignored),
            indicated_file_name.clone(),
            name_style_by_metadata(metadata),
        )
    };

    FileInfo {
        file_type: details.file_type,
        mode: details.mode,
        mode_bits: details.mode_bits,
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

/// Return the displayed name, preserving special styling for `.` and `..`.
pub fn check_display_name(info: &FileInfo) -> String {
    match &info.full_path.to_string_lossy() {
        p if p.ends_with("/.") => ".".blue().to_string(),
        p if p.ends_with("/..") => "..".blue().to_string(),
        _ => info.display_name.to_string(),
    }
}

fn entry_name_is_hidden(name: &OsStr) -> bool {
    platform::entry_name_is_hidden(name)
}

fn sort_key(name: &OsStr) -> Vec<u8> {
    platform::sort_key(name)
}

/// Escape control characters so paths cannot inject terminal control output.
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

pub(crate) fn sanitize_path_for_terminal(path: &Path) -> String {
    sanitize_for_terminal(&path.to_string_lossy())
}

/// Append the configured file-type indicator to a sanitized entry name.
fn format_name_with_indicator(
    safe_name: &str,
    metadata: &fs::Metadata,
    params: &Params,
) -> String {
    format!("{safe_name}{}", indicator_suffix(metadata, params))
}

/// Return the suffix for the configured indicator style.
///
/// Long-format symlink rows display targets with `->`, so the short-format
/// `@` symlink marker is suppressed there.
fn indicator_suffix(metadata: &fs::Metadata, params: &Params) -> &'static str {
    if metadata.is_symlink() && params.long_format {
        return "";
    }

    match params.indicator_style {
        IndicatorStyle::None => "",
        IndicatorStyle::Slash => {
            if metadata.is_dir() {
                "/"
            } else {
                ""
            }
        }
        IndicatorStyle::FileType => {
            file_type_indicator_suffix(metadata, false)
        }
        IndicatorStyle::Classify => file_type_indicator_suffix(metadata, true),
    }
}

/// Return the GNU-style indicator suffix for the entry metadata.
fn file_type_indicator_suffix(
    metadata: &fs::Metadata,
    classify_executables: bool,
) -> &'static str {
    file_type_indicator_suffix_for_type(
        platform::metadata_file_type(metadata),
        classify_executables,
        platform::is_executable(metadata),
    )
}

pub(crate) fn file_type_indicator_suffix_for_type(
    file_type: LongFormatFileType,
    classify_executables: bool,
    executable: bool,
) -> &'static str {
    match file_type {
        LongFormatFileType::Directory => "/",
        LongFormatFileType::Symlink => "@",
        LongFormatFileType::Fifo => "|",
        LongFormatFileType::Socket => "=",
        LongFormatFileType::Regular if classify_executables && executable => {
            "*"
        }
        LongFormatFileType::Regular
        | LongFormatFileType::CharDevice
        | LongFormatFileType::BlockDevice
        | LongFormatFileType::Unknown => "",
    }
}

fn apply_dim(style: StyledText, dimmed: bool) -> StyledText {
    if dimmed { style.dim() } else { style }
}

fn plain_text(text: impl Into<String>, dimmed: bool) -> String {
    apply_dim(StyledText::plain(text), dimmed).to_string()
}

fn name_style_by_metadata(metadata: &fs::Metadata) -> NameStyle {
    platform::name_style_by_metadata(metadata)
}

fn colorize_name_by_metadata(
    safe_name: &str,
    metadata: &fs::Metadata,
    dimmed: bool,
) -> String {
    match name_style_by_metadata(metadata) {
        NameStyle::Symlink => apply_dim(safe_name.cyan(), dimmed).to_string(),
        NameStyle::Directory => {
            apply_dim(safe_name.blue(), dimmed).to_string()
        }
        NameStyle::Executable => {
            apply_dim(safe_name.green().bold(), dimmed).to_string()
        }
        NameStyle::Socket => {
            apply_dim(safe_name.magenta().bold(), dimmed).to_string()
        }
        NameStyle::Fifo => apply_dim(safe_name.yellow(), dimmed).to_string(),
        NameStyle::CharDevice | NameStyle::BlockDevice => {
            apply_dim(safe_name.yellow().bold(), dimmed).to_string()
        }
        NameStyle::Plain => plain_text(safe_name, dimmed),
    }
}

/// Format a symlink name, optionally including and styling its target.
pub(crate) fn format_symlink_display_name_with_dim(
    source_name: &str,
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
                        apply_dim(source_name.cyan(), dimmed),
                        plain_text(" -> ", dimmed),
                        display_target
                    )
                } else {
                    format!(
                        "{}{}{}{}{}",
                        apply_dim(source_name.cyan(), dimmed),
                        plain_text(" -> ", dimmed),
                        display_target,
                        plain_text(" ", dimmed),
                        apply_dim("[Broken Link]".red(), dimmed)
                    )
                }
            } else {
                apply_dim(source_name.cyan(), dimmed).to_string()
            }
        }
        Err(_) => {
            if params.long_format {
                apply_dim(
                    format!("{source_name} -> (unreadable)").red(),
                    dimmed,
                )
                .to_string()
            } else {
                apply_dim(source_name.cyan(), dimmed).to_string()
            }
        }
    }
}

/// Format a path-related IO error for terminal output.
pub(crate) fn format_path_error(path: &Path, err: &io::Error) -> String {
    format!("lsplus: {}: {}", sanitize_path_for_terminal(path), err)
}

fn report_path_error(path: &Path, err: &io::Error) {
    eprintln!("{}", format_path_error(path, err));
}
