//! Runtime orchestration for listing paths and rendering output.
//!
//! This module bridges parsed CLI flags, config parameters, glob expansion,
//! filesystem metadata collection, and the selected output renderer.

use glob::glob;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::Params;
use crate::cli;
use crate::settings;
use crate::structs::FileInfo;
use crate::utils;
use crate::utils::file::{
    collect_file_info, create_file_info,
    create_file_info_from_metadata_with_gitignore,
    create_file_info_with_gitignore, format_path_error, sanitize_for_terminal,
    sanitize_path_for_terminal as display_path,
};
use crate::utils::gitignore::GitignoreCache;

#[derive(Debug)]
pub(crate) struct ListingSection {
    pub(crate) header: Option<String>,
    pub(crate) entries: Vec<FileInfo>,
}

#[derive(Debug)]
pub(crate) struct TreeSection {
    pub(crate) header: String,
    pub(crate) entries: Vec<TreeEntry>,
}

#[derive(Debug)]
pub(crate) struct TreeEntry {
    pub(crate) info: FileInfo,
    pub(crate) name_prefix: String,
}

struct RecursiveDirectory {
    section: ListingSection,
    children: Vec<PathBuf>,
}

/// Run `lsplus` using parsed CLI flags and config loaded from disk.
pub fn run_with_flags(args: cli::Flags) -> io::Result<()> {
    let config = settings::load_config();
    run_with_flags_and_config(args, &config)
}

/// Run `lsplus` using parsed CLI flags and an explicit config value.
///
/// This is primarily useful in tests and library-style entry points that want
/// to inject config without relying on filesystem state.
pub fn run_with_flags_and_config(
    args: cli::Flags,
    config: &Params,
) -> io::Result<()> {
    let params = Params::merge(&args, config);
    if params.recursive && params.tree {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--tree and --recursive cannot be used together",
        ));
    }
    utils::color::configure_color_output(&params);
    let patterns = patterns_from_args(args.paths);

    run_multi(&patterns, &params)
}

/// Return explicit CLI paths or the default current-directory pattern.
pub(crate) fn patterns_from_args(paths: Vec<String>) -> Vec<String> {
    if paths.is_empty() {
        vec![String::from(".")]
    } else {
        paths
    }
}

fn run_multi(patterns: &[String], params: &Params) -> io::Result<()> {
    if params.tree {
        return render_tree_sections(
            &collect_tree_sections(patterns, params)?,
            params,
        );
    }

    if params.recursive {
        return render_recursive_listing(patterns, params);
    }

    let sections = collect_listing_sections(patterns, params)?;

    render_listing_sections(&sections, params)
}

fn render_listing_sections(
    sections: &[ListingSection],
    params: &Params,
) -> io::Result<()> {
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            writeln!(io::stdout())?;
        }

        if let Some(header) = &section.header {
            writeln!(io::stdout(), "{header}:")?;
        }

        if params.long_format {
            utils::render::display_long_format(&section.entries, params)?;
        } else {
            utils::render::display_short_format(&section.entries)?;
        }
    }

    Ok(())
}

fn render_tree_sections(
    sections: &[TreeSection],
    params: &Params,
) -> io::Result<()> {
    let show_headers = sections.len() > 1;

    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            writeln!(io::stdout())?;
        }

        if show_headers {
            writeln!(
                io::stdout(),
                "{}:",
                utils::render::directory_header_text(&section.header)
            )?;
        }
        utils::render::display_long_format_with_name_prefixes(
            section
                .entries
                .iter()
                .map(|entry| (&entry.info, entry.name_prefix.as_str())),
            params,
        )?;
    }

    Ok(())
}

fn render_recursive_listing(
    patterns: &[String],
    params: &Params,
) -> io::Result<()> {
    let operands = collect_operands(patterns, params)?;
    let (file_entries, directory_operands) =
        split_file_and_directory_operands(&operands, params)?;
    let mut rendered_section = false;

    if !file_entries.is_empty() {
        render_listing_section(
            &mut rendered_section,
            &ListingSection {
                header: None,
                entries: file_entries,
            },
            params,
        )?;
    }

    walk_recursive_operands(directory_operands, params, &mut |section| {
        render_listing_section(&mut rendered_section, &section, params)
    })?;

    Ok(())
}

fn walk_recursive_operands(
    directory_operands: Vec<&PathBuf>,
    params: &Params,
    sink: &mut impl FnMut(ListingSection) -> io::Result<()>,
) -> io::Result<()> {
    let mut first_error = None;

    for path in directory_operands {
        if let Err(err) = walk_recursive_directory(path, params, true, 1, sink)
        {
            report_path_error(path, &err);
            if first_error.is_none() {
                first_error = Some(err);
            }
        }
    }

    if let Some(err) = first_error {
        return Err(err);
    }

    Ok(())
}

fn walk_recursive_directory(
    path: &Path,
    params: &Params,
    fail_on_error: bool,
    visible_entry_depth: usize,
    sink: &mut impl FnMut(ListingSection) -> io::Result<()>,
) -> io::Result<()> {
    let mut directory = match collect_recursive_directory(
        path,
        params,
        visible_entry_depth > 1,
    ) {
        Ok(directory) => directory,
        Err(err) if fail_on_error => return Err(err),
        Err(err) => {
            report_path_error(path, &err);
            return Ok(());
        }
    };

    directory.section.header =
        recursive_section_header(path, visible_entry_depth);
    sink(directory.section)?;

    if params
        .recursive_level
        .is_some_and(|limit| visible_entry_depth >= limit)
    {
        return Ok(());
    }

    for child in directory.children {
        walk_recursive_directory(
            &child,
            params,
            false,
            visible_entry_depth + 1,
            sink,
        )?;
    }

    Ok(())
}

fn render_listing_section(
    rendered_section: &mut bool,
    section: &ListingSection,
    params: &Params,
) -> io::Result<()> {
    if *rendered_section {
        writeln!(io::stdout())?;
    }

    if let Some(header) = &section.header {
        writeln!(
            io::stdout(),
            "{}:",
            utils::render::directory_header_text(header)
        )?;
    }

    if params.long_format {
        utils::render::display_long_format(&section.entries, params)?;
    } else {
        utils::render::display_short_format(&section.entries)?;
    }

    *rendered_section = true;

    Ok(())
}

pub(crate) fn collect_listing_sections(
    patterns: &[String],
    params: &Params,
) -> io::Result<Vec<ListingSection>> {
    let operands = collect_operands(patterns, params)?;
    build_listing_sections(&operands, params)
}

pub(crate) fn collect_tree_sections(
    patterns: &[String],
    params: &Params,
) -> io::Result<Vec<TreeSection>> {
    let operands = collect_operands(patterns, params)?;
    Ok(build_tree_sections(&operands, params))
}

fn collect_operands(
    patterns: &[String],
    params: &Params,
) -> io::Result<Vec<PathBuf>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }

    let mut operands = Vec::new();

    for pattern in patterns {
        append_pattern_operands(&mut operands, pattern, params)?;
    }

    Ok(operands)
}

fn append_pattern_operands(
    operands: &mut Vec<PathBuf>,
    pattern: &str,
    _params: &Params,
) -> io::Result<()> {
    match glob(pattern) {
        Ok(entries) => {
            let start_len = operands.len();
            let mut had_entry_error = false;

            for entry in entries {
                match entry {
                    Ok(path) => operands.push(path),
                    Err(err) => {
                        had_entry_error = true;
                        eprintln!(
                            "lsplus: {}: {}",
                            sanitize_for_terminal(
                                &err.path().to_string_lossy()
                            ),
                            err.error()
                        );
                    }
                }
            }

            if operands.len() == start_len && !had_entry_error {
                eprintln!(
                    "lsplus: {}: No such file or directory",
                    sanitize_for_terminal(pattern)
                );
            }
        }
        Err(e) => eprintln!("lsplus: failed to read glob pattern: {}", e),
    }

    Ok(())
}

fn build_listing_sections(
    operands: &[PathBuf],
    params: &Params,
) -> io::Result<Vec<ListingSection>> {
    let (file_entries, directory_operands) =
        split_file_and_directory_operands(operands, params)?;

    let show_directory_headers = params.recursive
        || !file_entries.is_empty()
        || directory_operands.len() > 1;
    let mut sections = Vec::new();

    if !file_entries.is_empty() {
        sections.push(ListingSection {
            header: None,
            entries: file_entries,
        });
    }

    if params.recursive {
        walk_recursive_operands(directory_operands, params, &mut |section| {
            sections.push(section);
            Ok(())
        })?;
    } else {
        for path in directory_operands {
            sections.push(ListingSection {
                header: show_directory_headers.then(|| display_path(path)),
                entries: collect_file_info(path, params)?,
            });
        }
    }

    Ok(sections)
}

fn split_file_and_directory_operands<'a>(
    operands: &'a [PathBuf],
    params: &Params,
) -> io::Result<(Vec<FileInfo>, Vec<&'a PathBuf>)> {
    let mut file_entries = Vec::new();
    let mut directory_operands = Vec::new();

    for path in operands {
        if is_display_directory(path) {
            directory_operands.push(path);
        } else {
            file_entries.push(create_file_info(path, params)?);
        }
    }

    Ok((file_entries, directory_operands))
}

fn collect_recursive_directory(
    path: &Path,
    params: &Params,
    hide_dot_entries: bool,
) -> io::Result<RecursiveDirectory> {
    let child_names = utils::file::collect_file_names(path, params)?;
    let mut entries = Vec::new();
    let mut children = Vec::new();
    let mut gitignore_cache = GitignoreCache::default();

    for child_name in child_names {
        if hide_dot_entries
            && (child_name.as_str() == "." || child_name.as_str() == "..")
        {
            continue;
        }

        let child_path = path.join(&child_name);
        let metadata = match fs::symlink_metadata(&child_path) {
            Ok(metadata) => metadata,
            Err(err) => {
                report_path_error(&child_path, &err);
                continue;
            }
        };

        entries.push(create_file_info_from_metadata_with_gitignore(
            &child_path,
            &metadata,
            params,
            &mut gitignore_cache,
        ));

        if is_traversable_child_name(&child_name)
            && metadata.is_dir()
            && !metadata.is_symlink()
            && !should_prune_directory(&child_path, params)
        {
            children.push(child_path);
        }
    }

    Ok(RecursiveDirectory {
        section: ListingSection {
            header: Some(display_path(path)),
            entries,
        },
        children,
    })
}

fn build_tree_sections(
    operands: &[PathBuf],
    params: &Params,
) -> Vec<TreeSection> {
    let mut sections = Vec::new();

    for path in operands {
        let mut gitignore_cache = GitignoreCache::default();
        if is_display_directory(path) {
            let mut section = TreeSection {
                header: display_path(path),
                entries: Vec::new(),
            };
            append_tree_entries(
                &mut section,
                path,
                params,
                &mut gitignore_cache,
                1,
                String::new(),
            );
            sections.push(section);
        } else {
            match create_file_info_with_gitignore(
                path,
                params,
                &mut gitignore_cache,
            ) {
                Ok(info) => sections.push(TreeSection {
                    header: display_path(path),
                    entries: vec![TreeEntry {
                        info,
                        name_prefix: String::new(),
                    }],
                }),
                Err(err) => report_path_error(path, &err),
            }
        }
    }

    sections
}

fn append_tree_entries(
    section: &mut TreeSection,
    directory: &Path,
    params: &Params,
    gitignore_cache: &mut GitignoreCache,
    depth: usize,
    ancestor_prefix: String,
) {
    let child_names = match utils::file::collect_file_names(directory, params)
    {
        Ok(names) => names,
        Err(err) => {
            report_path_error(directory, &err);
            return;
        }
    };

    let child_names = traversal_child_names(child_names);
    let child_count = child_names.len();
    for (index, child_name) in child_names.iter().enumerate() {
        let child_path = directory.join(child_name);
        let is_last = index + 1 == child_count;
        let branch = if is_last { "└── " } else { "├── " };
        let name_prefix = if depth == 1 {
            String::new()
        } else {
            format!("{ancestor_prefix}{branch}")
        };

        match create_file_info_with_gitignore(
            &child_path,
            params,
            gitignore_cache,
        ) {
            Ok(info) => {
                section.entries.push(TreeEntry { info, name_prefix });
            }
            Err(err) => {
                report_path_error(&child_path, &err);
                continue;
            }
        }

        if depth < params.tree_level
            && is_recursable_directory(&child_path)
            && !should_prune_directory(&child_path, params)
        {
            let next_prefix = if depth == 1 {
                ""
            } else if is_last {
                "    "
            } else {
                "│   "
            };
            append_tree_entries(
                section,
                &child_path,
                params,
                gitignore_cache,
                depth + 1,
                format!("{ancestor_prefix}{next_prefix}"),
            );
        }
    }
}

fn traversal_child_names(child_names: Vec<String>) -> Vec<String> {
    child_names
        .into_iter()
        .filter(|name| is_traversable_child_name(name))
        .collect()
}

fn is_traversable_child_name(name: &str) -> bool {
    name != "." && name != ".."
}

/// Return whether an explicit operand should be listed as a directory.
///
/// This follows symlinks so a direct symlink-to-directory operand behaves like
/// `ls`: it gets its own directory listing.
fn is_display_directory(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => fs::metadata(path)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false),
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

/// Return whether traversal may descend into a discovered child directory.
///
/// Unlike [`is_display_directory`], this intentionally does not follow
/// symlinks. Recursive and tree traversal must not chase symlinked directories
/// because that can loop back into an ancestor.
fn is_recursable_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.is_dir() && !metadata.is_symlink())
        .unwrap_or(false)
}

fn should_prune_directory(path: &Path, params: &Params) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            params.prune_dirs.iter().any(|pruned| pruned == name)
        })
}

fn recursive_section_header(
    path: &Path,
    visible_entry_depth: usize,
) -> Option<String> {
    let header_path = path.strip_prefix(".").unwrap_or(path);
    if visible_entry_depth == 1 && header_path.as_os_str().is_empty() {
        None
    } else {
        Some(display_path(header_path))
    }
}

fn report_path_error(path: &Path, err: &io::Error) {
    eprintln!("{}", format_path_error(path, err));
}
