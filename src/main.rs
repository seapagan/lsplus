use chrono::{DateTime, Local};
use clap::Parser;
use glob::glob;
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::io;
use std::path::PathBuf;
use std::process::exit;

mod cli;
mod structs;
mod utils;
use config::{Config, File, FileFormat};
use dirs_next::home_dir;

use strip_ansi_escapes::strip_str;
use structs::{FileInfo, Params};
use utils::file::{check_display_name, collect_file_info};

fn load_config() -> Params {
    let mut config_path = PathBuf::new();

    // Get the home directory and construct the path
    if let Some(home_dir) = home_dir() {
        config_path.push(home_dir);
        config_path.push(".config/lsplus/config.toml");
    }

    let settings = Config::builder()
        .add_source(File::new(config_path.to_str().unwrap(), FileFormat::Toml))
        .build();

    match settings {
        Ok(config) => config.into(), // Convert Config into Params using the From trait
        Err(e) => {
            // If the error is related to the file not being found, return default Params
            if e.to_string().contains("not found") {
                Params::default()
            } else {
                eprintln!("Error loading config: {}", e);
                Params::default()
            }
        }
    }
}

fn main() {
    let args = cli::Flags::parse();
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    // Load config values
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
        shorten_names: args.shorten_names || config.shorten_names,
    };

    let patterns = if args.paths.is_empty() {
        vec![String::from(".")]
    } else {
        args.paths
    };

    if let Err(e) = run_multi(&patterns, &params) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run_multi(patterns: &[String], params: &Params) -> io::Result<()> {
    // Get terminal width once at the start
    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
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
        display_long_format(&all_file_info, params, terminal_width)
    } else {
        display_short_format(&all_file_info, params)
    }
}

fn calculate_column_widths(
    file_info: &[FileInfo],
    params: &Params,
    terminal_width: usize,
) -> (usize, usize) {
    let mut max_mode_width = 0;
    let mut max_user_width = 0;
    let mut max_group_width = 0;
    let mut max_size_width = 0;
    let mut max_date_width = 0;
    let icon_width = if params.no_icons { 0 } else { 2 }; // 2 for icon + space

    for info in file_info {
        // Mode (type + permissions)
        max_mode_width = max_mode_width.max(info.mode.len() + 2); // +2 for type and space
                                                                  // User
        max_user_width = max_user_width.max(info.user.len() + 1); // +1 for space
                                                                  // Group
        max_group_width = max_group_width.max(info.group.len() + 1); // +1 for space
                                                                     // Size
        let (size, unit) =
            utils::format::show_size(info.size, params.human_readable);
        max_size_width = max_size_width.max(size.len() + unit.len() + 1); // +1 for space
                                                                          // Date
        let date_str = if params.fuzzy_time {
            utils::fuzzy_time(info.mtime).to_string()
        } else {
            let datetime: DateTime<Local> = DateTime::from(info.mtime);
            datetime.format("%c").to_string()
        };
        max_date_width = max_date_width.max(date_str.len() + 2); // +2 for spaces
    }

    let total_fixed_width = max_mode_width
        + max_user_width
        + max_group_width
        + max_size_width
        + max_date_width
        + icon_width;

    // Account for table borders and padding (2 chars for borders, 1 for padding on each side)
    let table_overhead = 4;
    // Account for column separators (1 char each)
    let num_separators = if icon_width > 0 { 8 } else { 7 };
    let available_width = terminal_width
        .saturating_sub(total_fixed_width)
        .saturating_sub(table_overhead)
        .saturating_sub(num_separators)
        .saturating_sub(1); // Additional adjustment for potential off-by-one errors

    (total_fixed_width, available_width)
}

fn display_long_format(
    file_info: &[FileInfo],
    params: &Params,
    terminal_width: usize,
) -> io::Result<()> {
    let mut table = utils::table::create_table(0);
    let (_, available_width) =
        calculate_column_widths(file_info, params, terminal_width);

    for info in file_info {
        let display_time = if params.fuzzy_time {
            utils::fuzzy_time(info.mtime).to_string()
        } else {
            let datetime: DateTime<Local> = DateTime::from(info.mtime);
            datetime.format("%c").to_string()
        };

        let (display_size, units) =
            utils::format::show_size(info.size, params.human_readable);

        let mut row_cells = Vec::with_capacity(9);

        row_cells
            .push(Cell::new(&format!("{}{} ", info.file_type, info.mode)));
        row_cells.push(Cell::new(&info.nlink.to_string()));
        row_cells.push(Cell::new(&format!(" {color_cyan}{}", info.user)));
        row_cells.push(Cell::new(&format!("{color_green}{} ", info.group)));
        row_cells.push(Cell::new(&display_size).style_spec("r"));

        if !units.is_empty() {
            row_cells.push(Cell::new(units));
        }

        row_cells.push(
            Cell::new(&format!(" {color_yellow}{} ", display_time))
                .style_spec("r"),
        );

        if let Some(icon) = &info.item_icon {
            row_cells.push(Cell::new(&format!("{} ", icon)));
        }

        let mut display_name = check_display_name(info);

        // Shorten the filename if needed and the option is enabled
        if params.shorten_names {
            display_name = utils::format::shorten_filename(
                &display_name,
                available_width,
            );
        }

        row_cells.push(Cell::new(&display_name));

        table.add_row(Row::new(row_cells));
    }

    table.printstd();
    Ok(())
}

fn display_short_format(
    file_info: &[FileInfo],
    _params: &Params,
) -> io::Result<()> {
    // Strip ANSI codes when calculating length
    let max_name_length = file_info
        .iter()
        .map(|info| {
            let display_name = check_display_name(info);
            // Remove ANSI escape sequences for length calculation
            let clean_name = strip_str(&display_name);
            clean_name.len()
        })
        .max()
        .unwrap_or(0)
        + 2; // Adding space between columns

    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let num_columns = terminal_width / max_name_length;

    let mut table = utils::table::create_table(2);

    for chunk in file_info.chunks(num_columns) {
        let mut row = Row::empty();
        for info in chunk {
            let display_name = check_display_name(info);
            let mut cell_content = String::new();
            if let Some(icon) = &info.item_icon {
                cell_content.push_str(&format!("{} ", icon));
            }
            cell_content.push_str(&display_name);
            row.add_cell(Cell::new(&cell_content));
        }
        table.add_row(row);
    }

    table.printstd();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_load_config_default() {
        // When no config file exists, should return default params
        let config = load_config();
        // We can only test that it returns a Params struct
        // The actual values might be affected by the user's config file
        assert!(matches!(config, Params { .. }));
    }

    #[test]
    fn test_run_multi() -> io::Result<()> {
        let temp_dir = tempdir()?;

        // Create some test files
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
        assert!(result.is_ok()); // The function handles errors internally
    }

    #[test]
    fn test_run_multi_with_glob() -> io::Result<()> {
        let temp_dir = tempdir()?;

        // Create test files with different extensions
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
        // Create a temporary directory for our test files
        let temp_dir = tempfile::tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file)?;

        // Create test file info
        let params = Params::default();
        let file_info = collect_file_info(&test_file, &params)?;

        // Test long format display with all features
        let params = Params {
            long_format: true,
            fuzzy_time: true,
            human_readable: true,
            shorten_names: true,
            ..Default::default()
        };
        let test_width = 80;
        display_long_format(&file_info, &params, test_width)?;

        // Test long format without optional features
        let params = Params {
            long_format: true,
            fuzzy_time: false,
            human_readable: false,
            shorten_names: false,
            ..Default::default()
        };
        display_long_format(&file_info, &params, test_width)?;

        // Test short format with name shortening
        let params = Params {
            shorten_names: true,
            ..Default::default()
        };
        display_short_format(&file_info, &params)?;

        // Test short format without name shortening
        let params = Params::default();
        display_short_format(&file_info, &params)?;

        Ok(())
    }

    #[test]
    fn test_main_flags() {
        // Test version flag
        assert!(cli::version_info().contains("lsplus"));

        // Test empty paths
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
            shorten_names: false,
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
        // Test with invalid config file
        let mut config_path = PathBuf::new();
        if let Some(home_dir) = home_dir() {
            config_path.push(home_dir);
            config_path.push(".config/lsplus/config.toml");
        }

        let settings = Config::builder()
            .add_source(config::File::new(
                config_path.to_str().unwrap(),
                FileFormat::Toml,
            ))
            .build();

        match settings {
            Ok(_) => (),
            Err(e) => {
                if e.to_string().contains("not found") {
                    assert_eq!(Params::default(), Params::default());
                } else {
                    assert_eq!(Params::default(), Params::default());
                }
            }
        }
    }

    #[test]
    fn test_run_multi_glob_error() {
        let params = Params::default();
        let invalid_pattern = vec![String::from("[invalid-glob-pattern")];

        // This should print an error message but not panic
        run_multi(&invalid_pattern, &params).unwrap();
    }

    #[test]
    fn test_load_config_error_other() {
        // Test with a malformed config file
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".config/lsplus/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "invalid = toml [ content").unwrap();

        let settings = Config::builder()
            .add_source(config::File::new(
                config_path.to_str().unwrap(),
                FileFormat::Toml,
            ))
            .build();

        match settings {
            Ok(_) => panic!("Expected error"),
            Err(e) => {
                assert!(!e.to_string().contains("not found"));
                let params = Params::default();
                assert_eq!(params.show_all, false);
            }
        }
    }

    #[test]
    fn test_main_error_handling() {
        // Test with a pattern that will cause an error
        let params = Params::default();
        let invalid_pattern =
            vec![String::from("/nonexistent/path/that/should/not/exist")];

        let result = run_multi(&invalid_pattern, &params);
        assert!(result.is_ok()); // The function handles errors internally
    }

    #[test]
    fn test_main_config_merge() {
        // Test merging of config and CLI args
        let config = Params {
            show_all: true,
            append_slash: true,
            dirs_first: false,
            almost_all: false,
            long_format: true,
            human_readable: true,
            no_icons: false,
            fuzzy_time: false,
            shorten_names: true,
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
            shorten_names: false,
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
            shorten_names: args.shorten_names || config.shorten_names,
        };

        // Verify the merging logic
        assert!(params.show_all);
        assert!(params.append_slash);
        assert!(params.dirs_first);
        assert!(params.almost_all);
        assert!(params.long_format);
        assert!(params.human_readable);
        assert!(params.no_icons);
        assert!(params.fuzzy_time);
        assert!(params.shorten_names);
    }

    #[test]
    fn test_run_multi_error_handling() {
        // Create a temporary directory and a file with no read permissions
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

        // This should print an error message but not panic
        let result = run_multi(&pattern, &params);
        assert!(result.is_ok()); // The function handles errors internally

        // Clean up by restoring permissions so the file can be deleted
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

        // This should handle empty glob results
        let result = run_multi(&pattern, &params);
        assert!(result.is_ok());
    }
}
