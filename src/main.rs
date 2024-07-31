use chrono::{DateTime, Local};
use clap::Parser;
use glob::glob;
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
mod cli;
mod utils;

struct Params {
    show_all: bool,
    append_slash: bool,
    dirs_first: bool,
    almost_all: bool,
    long_format: bool,
    human_readable: bool,
    no_icons: bool,
    fuzzy_time: bool,
}

fn main() {
    let args = cli::Flags::parse();
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    let params = Params {
        show_all: args.show_all,
        append_slash: args.slash,
        dirs_first: args.dirs_first,
        almost_all: args.almost_all,
        long_format: args.long,
        human_readable: args.human_readable,
        no_icons: args.no_icons,
        fuzzy_time: args.fuzzy_time,
    };

    let pattern = &args.path;
    let paths: Vec<PathBuf> = match glob(pattern) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(e) => {
            eprintln!("Failed to read glob pattern: {}", e);
            exit(1);
        }
    };

    if paths.is_empty() {
        // If no files match, just run with the original pattern
        // This allows the program to handle errors for non-existent paths
        if let Err(e) = run(pattern, &params) {
            handle_error(pattern, e);
        }
    } else {
        for path in paths {
            if let Err(e) = run(&path.to_string_lossy(), &params) {
                handle_error(&path.to_string_lossy(), e);
            }
        }
    }
}

fn handle_error(path: &str, e: io::Error) {
    let error_message = match e.kind() {
        io::ErrorKind::PermissionDenied => "Permission denied",
        io::ErrorKind::NotFound => "No such file or directory",
        _ => &e.to_string(),
    };
    eprintln!("lsp: {}: {}", path, error_message);
}

fn run(path: &str, params: &Params) -> io::Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}': No such file or directory", path.display()),
        ));
    }

    if params.long_format {
        display_long_format(path, params)
    } else {
        display_short_format(path, params)
    }
}

fn display_long_format(path: &Path, params: &Params) -> io::Result<()> {
    let mut table = utils::table::create_table(0);
    let file_names = utils::file::collect_file_names(path, params)?;

    for file_name in file_names {
        let path_metadata = fs::symlink_metadata(path)?;

        let full_path = if path_metadata.is_dir() {
            Path::new(path).join(&file_name)
        } else {
            PathBuf::from(file_name.clone())
        };
        let metadata = fs::symlink_metadata(&full_path)?;
        let item_icon = utils::icons::get_item_icon(
            &metadata,
            &full_path.to_string_lossy(),
        );
        let (file_type, mode, nlink, size, mtime, user, group, executable) =
            utils::file::get_file_details(&metadata);

        let mut display_name = file_name.clone();
        if metadata.is_symlink() {
            match fs::read_link(&full_path) {
                Ok(target) => {
                    let target_path = if target.is_relative() {
                        full_path.parent().unwrap().join(target)
                    } else {
                        target
                    };
                    if target_path.exists() {
                        display_name = format!(
                            "{color_cyan}{} -> {}",
                            file_name,
                            target_path.display()
                        );
                    } else {
                        display_name = format!(
                            "{color_cyan}{} -> {} {color_red}[Broken Link]",
                            file_name,
                            target_path.display()
                        );
                    }
                }
                Err(_) => {
                    display_name =
                        format!("{color_red}{} -> (unreadable)", file_name);
                }
            }
        } else if metadata.is_dir() {
            display_name = format!("{color_blue}{}", file_name);
        } else if executable {
            // this is an executable file, but not a folder or symlink
            display_name =
                format!("{style_bold}{color_green}{}", display_name);
        }

        let display_time: String = if params.fuzzy_time {
            utils::fuzzy_time(mtime).to_string()
        } else {
            let datetime: DateTime<Local> = DateTime::from(mtime);
            datetime.format("%c").to_string()
        };

        let (display_size, units) =
            utils::format::show_size(size, params.human_readable);

        let mut row_cells = Vec::with_capacity(9);

        row_cells.push(Cell::new(&format!("{}{} ", file_type, mode)));
        row_cells.push(Cell::new(&nlink.to_string()));
        row_cells.push(Cell::new(&format!(" {color_cyan}{}", user)));
        row_cells.push(Cell::new(&format!("{color_green}{} ", group)));
        row_cells.push(Cell::new(&display_size).style_spec("r"));

        if !units.is_empty() {
            row_cells.push(Cell::new(units));
        }

        row_cells.push(
            Cell::new(&format!(" {color_yellow}{} ", display_time))
                .style_spec("r"),
        );

        if !params.no_icons {
            row_cells.push(Cell::new(&format!("{} ", item_icon)));
        }

        row_cells.push(Cell::new(&display_name.to_string()));

        table.add_row(Row::new(row_cells));
    }
    table.printstd();
    Ok(())
}

fn display_short_format(path: &Path, params: &Params) -> io::Result<()> {
    let file_names = utils::file::collect_file_names(path, params)?;
    let max_name_length = utils::file::calculate_max_name_length(&file_names);
    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let num_columns = terminal_width / max_name_length;

    let mut table = utils::table::create_table(2);
    utils::table::add_files_to_table(&mut table, &file_names, num_columns);

    table.printstd();
    Ok(())
}
