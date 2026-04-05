use glob::glob;
use std::io;
use std::path::PathBuf;

use crate::Params;
use crate::cli;
use crate::settings;
use crate::utils;
use crate::utils::file::collect_file_info;

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
            let paths: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
            if paths.is_empty() {
                eprintln!("lsplus: {}: No such file or directory", pattern);
            } else {
                append_paths(all_file_info, &paths, params)?;
            }
        }
        Err(e) => eprintln!("Failed to read glob pattern: {}", e),
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
