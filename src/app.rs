use glob::glob;
use std::io;
use std::path::PathBuf;

use crate::Params;
use crate::cli;
use crate::settings;
use crate::utils;
use crate::utils::file::collect_file_info;

pub fn run_with_flags(args: cli::Flags) -> io::Result<()> {
    let config = settings::load_config();
    let params = Params::merge(&args, &config);
    let patterns = patterns_from_args(args.paths);

    run_multi(&patterns, &params)
}

fn patterns_from_args(paths: Vec<String>) -> Vec<String> {
    if paths.is_empty() {
        vec![String::from(".")]
    } else {
        paths
    }
}

fn run_multi(patterns: &[String], params: &Params) -> io::Result<()> {
    let mut all_file_info = Vec::new();

    for pattern in patterns {
        append_pattern_matches(&mut all_file_info, pattern, params)?;
    }

    if params.long_format {
        utils::render::display_long_format(&all_file_info, params)
    } else {
        utils::render::display_short_format(&all_file_info)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_run_multi() -> io::Result<()> {
        let temp_dir = tempdir()?;

        File::create(temp_dir.path().join("test1.txt"))?;
        File::create(temp_dir.path().join("test2.txt"))?;
        std::fs::create_dir(temp_dir.path().join("testdir"))?;

        let params = Params::default();
        let patterns = vec![temp_dir.path().to_string_lossy().to_string()];

        assert!(run_multi(&patterns, &params).is_ok());
        Ok(())
    }

    #[test]
    fn test_run_multi_nonexistent() {
        let params = Params::default();
        let patterns = vec![String::from("/nonexistent/path")];

        let result = run_multi(&patterns, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_append_pattern_matches_with_glob() -> io::Result<()> {
        let temp_dir = tempdir()?;

        File::create(temp_dir.path().join("test1.txt"))?;
        File::create(temp_dir.path().join("test2.txt"))?;
        File::create(temp_dir.path().join("test.rs"))?;

        let params = Params::default();
        let pattern = format!("{}/*.txt", temp_dir.path().to_string_lossy());
        let mut file_info = Vec::new();

        append_pattern_matches(&mut file_info, &pattern, &params)?;

        assert_eq!(file_info.len(), 2);
        Ok(())
    }

    #[test]
    fn test_patterns_from_args_defaults_to_current_directory() {
        assert_eq!(patterns_from_args(Vec::new()), vec![String::from(".")]);
    }

    #[test]
    fn test_patterns_from_args_preserves_explicit_paths() {
        let paths = vec![String::from("left"), String::from("right")];

        assert_eq!(patterns_from_args(paths.clone()), paths);
    }

    #[test]
    fn test_run_multi_glob_error() {
        let params = Params::default();
        let invalid_pattern = vec![String::from("[invalid-glob-pattern")];

        run_multi(&invalid_pattern, &params).unwrap();
    }

    #[test]
    fn test_append_pattern_matches_with_empty_glob() -> io::Result<()> {
        let params = Params::default();
        let mut file_info = Vec::new();

        append_pattern_matches(
            &mut file_info,
            "**/nonexistent_pattern_*.xyz",
            &params,
        )?;

        assert!(file_info.is_empty());
        Ok(())
    }

    #[test]
    fn test_run_multi_error_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("no_read.txt");
        std::fs::write(&test_file, "test").unwrap();
        std::fs::set_permissions(
            &test_file,
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let params = Params::default();
        let pattern = vec![test_file.to_string_lossy().to_string()];

        let result = run_multi(&pattern, &params);
        assert!(result.is_ok());

        std::fs::set_permissions(
            &test_file,
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
    }

    #[test]
    fn test_run_multi_empty_pattern() {
        let params = Params::default();
        let pattern = vec![String::from("**/nonexistent_pattern_*.xyz")];

        let result = run_multi(&pattern, &params);
        assert!(result.is_ok());
    }
}
