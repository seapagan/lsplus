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
        display_long_format(&all_file_info, params)
    } else {
        display_short_format(&all_file_info, params)
    }
}

// fn handle_error(path: &str, e: io::Error) {
//     let error_message = match e.kind() {
//         io::ErrorKind::PermissionDenied => "Permission denied",
//         io::ErrorKind::NotFound => "No such file or directory",
//         _ => &e.to_string(),
//     };
//     eprintln!("lsp: {}: {}", path, error_message);
// }

fn display_long_format(
    file_info: &[FileInfo],
    params: &Params,
) -> io::Result<()> {
    let mut table = utils::table::create_table(0);

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

        let display_name = check_display_name(info);

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
    let max_name_length = file_info
        .iter()
        .map(|info| info.display_name.len())
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
