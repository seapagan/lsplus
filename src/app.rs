//! Runtime orchestration for listing paths and rendering output.
//!
//! This module bridges parsed CLI flags, config parameters, glob expansion,
//! filesystem metadata collection, and the selected output renderer.

use glob::glob;
use std::io;
use std::path::PathBuf;

use crate::Params;
use crate::cli;
use crate::settings;
use crate::utils;
use crate::utils::file::{collect_file_info, sanitize_for_terminal};

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
    let all_file_info = collect_matches(patterns, params)?;

    if params.long_format {
        utils::render::display_long_format(&all_file_info, params)
    } else {
        utils::render::display_short_format(&all_file_info)
    }
}

/// Expand path patterns and collect display data for all matching entries.
///
/// `collect_matches` reports missing patterns to stderr, skips them, and
/// continues with other patterns.
pub(crate) fn collect_matches(
    patterns: &[String],
    params: &Params,
) -> io::Result<Vec<crate::FileInfo>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_file_info = Vec::new();

    for pattern in patterns {
        append_pattern_matches(&mut all_file_info, pattern, params)?;
    }

    Ok(all_file_info)
}

fn append_pattern_matches(
    all_file_info: &mut Vec<crate::FileInfo>,
    pattern: &str,
    params: &Params,
) -> io::Result<()> {
    match glob(pattern) {
        Ok(entries) => {
            let mut paths: Vec<PathBuf> = Vec::new();
            let mut had_entry_error = false;

            for entry in entries {
                match entry {
                    Ok(path) => paths.push(path),
                    Err(err) => {
                        had_entry_error = true;
                        eprintln!(
                            "lsplus: {}: {}",
                            sanitize_for_terminal(pattern),
                            err
                        );
                    }
                }
            }

            if paths.is_empty() {
                if !had_entry_error {
                    eprintln!(
                        "lsplus: {}: No such file or directory",
                        sanitize_for_terminal(pattern)
                    );
                }
            } else {
                append_paths(all_file_info, &paths, params)?;
            }
        }
        Err(e) => eprintln!("lsplus: failed to read glob pattern: {}", e),
    }

    Ok(())
}

fn append_paths(
    all_file_info: &mut Vec<crate::FileInfo>,
    paths: &[PathBuf],
    params: &Params,
) -> io::Result<()> {
    for path in paths {
        let file_info = collect_file_info(path, params)?;
        all_file_info.extend(file_info);
    }

    Ok(())
}
