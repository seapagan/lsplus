//! Filesystem inspection and entry-name formatting.
//!
//! This module collects the metadata needed by renderers, applies visibility
//! and directory-ordering rules, sanitizes terminal output, and prepares
//! styled names for regular files, directories, symlinks, and gitignored
//! entries.

use colored_text::{Colorize, StyledText};
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::IndicatorStyle;
use crate::Params;
use crate::platform::{self, EntryClassification, LongFormatFileType};
use crate::structs::FileInfo;
use crate::structs::NameStyle;
use crate::utils::{self, gitignore::GitignoreCache};

/// Directory entry data captured before visibility filtering and sorting.
pub(crate) struct DirectoryEntryData {
    /// Raw entry name from `read_dir`.
    pub file_name: OsString,
    /// Full entry path.
    pub path: PathBuf,
    /// Platform classification captured while reading the entry.
    ///
    /// An error means metadata acquisition failed before classification.
    pub classification_result: Result<EntryClassification, io::Error>,
}

/// Return displayable names for a file path or visible entries in a directory.
pub fn collect_file_names(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let symlink_metadata = fs::symlink_metadata(path)?;
    let classification = platform::classify_entry(path, &symlink_metadata);

    if !classification.display_as_directory {
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
                    classification_result: fs::symlink_metadata(entry.path())
                        .map(|metadata| {
                            platform::classify_entry(&entry.path(), &metadata)
                        }),
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
                let visible = match &entry.classification_result {
                    Ok(classification) => !classification.hidden,
                    Err(_) => {
                        !platform::entry_name_is_hidden(&entry.file_name)
                    }
                };
                if params.show_all || params.almost_all || visible {
                    visible_entries.push(entry);
                }
            }
            Err(err) => report_path_error(path, &err),
        }
    }

    visible_entries.sort_by(|left, right| {
        platform::compare_entry_names(&left.file_name, &right.file_name)
    });

    if params.dirs_first {
        let (dirs, files): (Vec<_>, Vec<_>) = visible_entries
            .into_iter()
            .partition(|entry| match &entry.classification_result {
                Ok(classification) => classification.group_with_directories,
                Err(err) => {
                    report_path_error(&entry.path, err);
                    false
                }
            });

        visible_entries = dirs.into_iter().chain(files).collect();
    }

    let mut file_names = Vec::new();
    file_names.extend(
        platform::synthetic_dot_entries(params)
            .iter()
            .map(|entry| (*entry).to_string()),
    );

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

    let classification = platform::classify_entry(path, &symlink_metadata);

    if classification.display_as_directory {
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
            Ok(mut info) => {
                if matches!(file_name.as_str(), "." | "..") {
                    info.short_name.clone_from(file_name);
                    info.display_name.clone_from(file_name);
                }
                file_info.push(info);
            }
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
    let classification = platform::classify_entry(path, metadata);
    let item_icon = if params.no_icons {
        None
    } else {
        Some(utils::icons::get_item_icon(classification.file_type, path))
    };
    let details = platform::file_details(path, metadata, classification);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    let safe_file_name = sanitize_for_terminal(&file_name);
    let indicated_file_name = format_name_with_indicator(
        &safe_file_name,
        path,
        metadata,
        classification,
        params,
    );

    let ignored = params.gitignore
        && gitignore_cache.is_ignored(path, metadata.is_dir());

    let name_style = platform::name_style(path, metadata, classification);
    let (display_name, short_name) = if classification.may_render_link_target {
        (
            format_symlink_display_name_with_dim(
                &indicated_file_name,
                path,
                fs::read_link(path),
                params,
                name_style,
                ignored,
            ),
            indicated_file_name.clone(),
        )
    } else {
        (
            colorize_name_by_metadata(
                &indicated_file_name,
                path,
                metadata,
                classification,
                ignored,
            ),
            indicated_file_name.clone(),
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
    match info.short_name.as_str() {
        "." => ".".blue().to_string(),
        ".." => "..".blue().to_string(),
        _ => info.display_name.to_string(),
    }
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
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
    params: &Params,
) -> String {
    format!(
        "{safe_name}{}",
        indicator_suffix(path, metadata, classification, params)
    )
}

/// Return the suffix for the configured indicator style.
///
/// Long-format symlink rows display targets with `->`, so the short-format
/// `@` symlink marker is suppressed there.
fn indicator_suffix(
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
    params: &Params,
) -> &'static str {
    if classification.may_render_link_target && params.long_format {
        return "";
    }

    match params.indicator_style {
        IndicatorStyle::None => "",
        // Deliberately inspect the link object: directory links and junctions
        // have no suffix in slash-only mode.
        IndicatorStyle::Slash => slash_indicator_suffix(metadata.is_dir()),
        IndicatorStyle::FileType => {
            file_type_indicator_suffix(path, metadata, classification, false)
        }
        IndicatorStyle::Classify => {
            file_type_indicator_suffix(path, metadata, classification, true)
        }
    }
}

/// Return the slash-only indicator for the listed link object.
pub(crate) fn slash_indicator_suffix(is_directory: bool) -> &'static str {
    if is_directory { "/" } else { "" }
}

/// Return the GNU-style indicator suffix for the entry metadata.
fn file_type_indicator_suffix(
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
    classify_executables: bool,
) -> &'static str {
    file_type_indicator_suffix_for_type(
        classification.file_type,
        classify_executables,
        platform::is_executable(path, metadata),
    )
}

pub(crate) fn file_type_indicator_suffix_for_type(
    file_type: LongFormatFileType,
    classify_executables: bool,
    executable: bool,
) -> &'static str {
    match file_type {
        LongFormatFileType::Directory => "/",
        LongFormatFileType::Symlink
        | LongFormatFileType::SymlinkFile
        | LongFormatFileType::SymlinkDirectory
        | LongFormatFileType::Junction => "@",
        LongFormatFileType::Fifo => "|",
        LongFormatFileType::Socket => "=",
        LongFormatFileType::Regular if classify_executables && executable => {
            "*"
        }
        LongFormatFileType::Regular
        | LongFormatFileType::CharDevice
        | LongFormatFileType::BlockDevice
        | LongFormatFileType::ReparsePoint
        | LongFormatFileType::Unknown => "",
    }
}

fn apply_dim(style: StyledText, dimmed: bool) -> StyledText {
    if dimmed { style.dim() } else { style }
}

fn plain_text(text: impl Into<String>, dimmed: bool) -> String {
    apply_dim(StyledText::plain(text), dimmed).to_string()
}

fn colorize_name_by_metadata(
    safe_name: &str,
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
    dimmed: bool,
) -> String {
    match platform::name_style(path, metadata, classification) {
        NameStyle::Symlink => apply_dim(safe_name.cyan(), dimmed).to_string(),
        NameStyle::Junction => {
            apply_dim(safe_name.magenta(), dimmed).to_string()
        }
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
    source_style: NameStyle,
    dimmed: bool,
) -> String {
    match target {
        Ok(target) => {
            let target_path = if target.is_relative() {
                path.parent().unwrap_or(Path::new("")).join(target)
            } else {
                target
            };
            let target_path = platform::normalize_path(target_path);
            let display_target = sanitize_path_for_terminal(&target_path);
            if params.long_format {
                let display_target = fs::symlink_metadata(&target_path)
                    .map(|metadata| {
                        let classification =
                            platform::classify_entry(&target_path, &metadata);
                        colorize_name_by_metadata(
                            &display_target,
                            &target_path,
                            &metadata,
                            classification,
                            dimmed,
                        )
                    })
                    .unwrap_or_else(|_| plain_text(&display_target, dimmed));

                let target_exists = target_path.try_exists();
                if matches!(target_exists, Ok(true)) {
                    format!(
                        "{}{}{}",
                        link_source_text(source_name, source_style, dimmed),
                        plain_text(" -> ", dimmed),
                        display_target
                    )
                } else if matches!(target_exists, Ok(false)) {
                    format!(
                        "{}{}{}{}{}",
                        link_source_text(source_name, source_style, dimmed),
                        plain_text(" -> ", dimmed),
                        display_target,
                        plain_text(" ", dimmed),
                        apply_dim("[Broken Link]".red(), dimmed)
                    )
                } else {
                    format!(
                        "{}{}{}{}{}",
                        link_source_text(source_name, source_style, dimmed),
                        plain_text(" -> ", dimmed),
                        display_target,
                        plain_text(" ", dimmed),
                        apply_dim("[Target Unresolved]".yellow(), dimmed)
                    )
                }
            } else {
                link_source_text(source_name, source_style, dimmed)
            }
        }
        Err(_) => {
            if params.long_format {
                apply_dim(
                    format!("{source_name} [Target Unavailable]").red(),
                    dimmed,
                )
                .to_string()
            } else {
                link_source_text(source_name, source_style, dimmed)
            }
        }
    }
}

fn link_source_text(
    source_name: &str,
    source_style: NameStyle,
    dimmed: bool,
) -> String {
    if source_style == NameStyle::Junction {
        apply_dim(source_name.magenta(), dimmed).to_string()
    } else {
        apply_dim(source_name.cyan(), dimmed).to_string()
    }
}

/// Format a path-related IO error for terminal output.
pub(crate) fn format_path_error(path: &Path, err: &io::Error) -> String {
    format!("lsplus: {}: {}", sanitize_path_for_terminal(path), err)
}

fn report_path_error(path: &Path, err: &io::Error) {
    eprintln!("{}", format_path_error(path, err));
}
