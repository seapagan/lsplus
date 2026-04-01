use glob::glob;
use std::io;
use std::path::PathBuf;

use config::{Config, File, FileFormat};
use dirs_next::home_dir;

pub mod cli;
pub mod structs;
pub mod utils;

pub use structs::{FileInfo, Params};

use utils::file::collect_file_info;

fn config_path() -> Option<PathBuf> {
    let mut path = home_dir()?;
    path.push(".config/lsplus/config.toml");
    Some(path)
}

fn load_config() -> Params {
    load_config_from_path(config_path())
}

fn load_config_from_path(config_path: Option<PathBuf>) -> Params {
    let Some(config_path) = config_path else {
        return Params::default();
    };

    if !config_path.is_file() {
        return Params::default();
    }

    let settings = Config::builder()
        .add_source(File::from(config_path).format(FileFormat::Toml))
        .build();

    match settings {
        Ok(config) => config.into(),
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            Params::default()
        }
    }
}

pub fn run_with_flags(args: cli::Flags) -> io::Result<()> {
    let config = load_config();

    let params = Params {
        show_all: args.show_all || config.show_all,
        append_slash: args.slash || config.append_slash,
        dirs_first: args.dirs_first || config.dirs_first,
        almost_all: args.almost_all || config.almost_all,
        long_format: args.long || config.long_format,
        human_readable: args.human_readable || config.human_readable,
        no_icons: args.no_icons || config.no_icons,
        fuzzy_time: args.fuzzy_time || config.fuzzy_time,
    };

    let patterns = if args.paths.is_empty() {
        vec![String::from(".")]
    } else {
        args.paths
    };

    run_multi(&patterns, &params)
}

fn run_multi(patterns: &[String], params: &Params) -> io::Result<()> {
    let mut all_file_info = Vec::new();

    for pattern in patterns {
        match glob(pattern) {
            Ok(entries) => {
                let paths: Vec<PathBuf> =
                    entries.filter_map(Result::ok).collect();
                if paths.is_empty() {
                    eprintln!(
                        "lsplus: {}: No such file or directory",
                        pattern
                    );
                } else {
                    for path in paths {
                        let file_info = collect_file_info(&path, params)?;
                        all_file_info.extend(file_info);
                    }
                }
            }
            Err(e) => eprintln!("Failed to read glob pattern: {}", e),
        }
    }

    if params.long_format {
        utils::render::display_long_format(&all_file_info, params)
    } else {
        utils::render::display_short_format(&all_file_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_load_config_default() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("missing-config.toml");

        let config = load_config_from_path(Some(config_path));

        assert_eq!(config, Params::default());
    }

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
    fn test_run_multi_with_glob() -> io::Result<()> {
        let temp_dir = tempdir()?;

        File::create(temp_dir.path().join("test1.txt"))?;
        File::create(temp_dir.path().join("test2.txt"))?;
        File::create(temp_dir.path().join("test.rs"))?;

        let params = Params::default();
        let pattern = format!("{}/*.txt", temp_dir.path().to_string_lossy());
        let patterns = vec![pattern];

        assert!(run_multi(&patterns, &params).is_ok());
        Ok(())
    }

    #[test]
    fn test_display_formats() -> io::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file)?;

        let params = Params::default();
        let file_info = collect_file_info(&test_file, &params)?;

        let params = Params {
            long_format: true,
            fuzzy_time: true,
            human_readable: true,
            ..Default::default()
        };
        utils::render::display_long_format(&file_info, &params)?;

        let params = Params {
            long_format: true,
            fuzzy_time: false,
            human_readable: false,
            ..Default::default()
        };
        utils::render::display_long_format(&file_info, &params)?;

        utils::render::display_short_format(&file_info)?;

        Ok(())
    }

    #[test]
    fn test_main_flags() {
        assert!(cli::version_info().contains("lsplus"));

        let args = cli::Flags {
            version: false,
            paths: vec![],
            show_all: false,
            almost_all: false,
            slash: false,
            dirs_first: false,
            long: false,
            human_readable: false,
            no_icons: false,
            fuzzy_time: false,
        };
        assert_eq!(
            if args.paths.is_empty() {
                vec![String::from(".")]
            } else {
                args.paths
            },
            vec![String::from(".")]
        );
    }

    #[test]
    fn test_load_config_error() {
        let config = load_config_from_path(None);

        assert_eq!(config, Params::default());
    }

    #[test]
    fn test_run_multi_glob_error() {
        let params = Params::default();
        let invalid_pattern = vec![String::from("[invalid-glob-pattern")];

        run_multi(&invalid_pattern, &params).unwrap();
    }

    #[test]
    fn test_load_config_error_other() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".config/lsplus/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "invalid = toml [ content").unwrap();

        let config = load_config_from_path(Some(config_path));

        assert_eq!(config, Params::default());
    }

    #[test]
    fn test_main_error_handling() {
        let params = Params::default();
        let invalid_pattern =
            vec![String::from("/nonexistent/path/that/should/not/exist")];

        let result = run_multi(&invalid_pattern, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_config_merge() {
        let config = Params {
            show_all: true,
            append_slash: true,
            dirs_first: false,
            almost_all: false,
            long_format: true,
            human_readable: true,
            no_icons: false,
            fuzzy_time: false,
        };

        let args = cli::Flags {
            version: false,
            paths: vec![],
            show_all: false,
            almost_all: true,
            slash: false,
            dirs_first: true,
            long: false,
            human_readable: false,
            no_icons: true,
            fuzzy_time: true,
        };

        let params = Params {
            show_all: args.show_all || config.show_all,
            append_slash: args.slash || config.append_slash,
            dirs_first: args.dirs_first || config.dirs_first,
            almost_all: args.almost_all || config.almost_all,
            long_format: args.long || config.long_format,
            human_readable: args.human_readable || config.human_readable,
            no_icons: args.no_icons || config.no_icons,
            fuzzy_time: args.fuzzy_time || config.fuzzy_time,
        };

        assert!(params.show_all);
        assert!(params.append_slash);
        assert!(params.dirs_first);
        assert!(params.almost_all);
        assert!(params.long_format);
        assert!(params.human_readable);
        assert!(params.no_icons);
        assert!(params.fuzzy_time);
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
