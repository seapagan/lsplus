use clap::Parser;
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::fs;
use std::io;
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
}

fn main() -> io::Result<()> {
    let args = cli::CLI::parse();
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    // read in the command line arguments
    let params = Params {
        show_all: args.show_all,
        append_slash: args.slash,
        dirs_first: args.dirs_first,
        almost_all: args.almost_all,
        long_format: args.long,
        human_readable: args.human_readable,
    };
    let path = args.path;

    // different behavior for long format or short format
    if params.long_format {
        let mut table = utils::create_table(0);
        let file_names = utils::collect_file_names(&path, &params)?;

        for file_name in file_names {
            let path_metadata = fs::symlink_metadata(&path)?;

            let full_path = if path_metadata.is_dir() {
                PathBuf::from(format!("{}/{}", path, file_name))
            } else {
                PathBuf::from(file_name.clone())
            };
            let metadata = fs::symlink_metadata(&full_path)?;
            let item_icon = utils::get_item_icon(&metadata);
            let (file_type, mode, nlink, size, mtime, user, group) =
                utils::get_file_details(&metadata);

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
                        display_name = format!(
                            "{color_red}{} -> (unreadable)",
                            file_name
                        );
                    }
                }
            } else if metadata.is_dir() {
                display_name = format!("{color_blue}{}", file_name);
            }

            let (display_size, units) =
                utils::show_size(size, params.human_readable);

            let mut row_cells = vec![
                Cell::new(&format!("{}{} ", file_type, mode)),
                Cell::new(&nlink.to_string()),
                Cell::new(&format!(" {color_cyan}{}", user)),
                Cell::new(&format!("{color_green}{} ", group)),
                Cell::new(&display_size).style_spec("r"),
                Cell::new(&format!(" {color_yellow}{} ", mtime)),
                Cell::new(&item_icon),
                Cell::new(&format!(" {}", display_name)),
            ];

            if !units.is_empty() {
                row_cells.insert(5, Cell::new(units)); //.style_spec("l"));
            }
            table.add_row(Row::new(row_cells));
        }
        table.printstd();
    } else {
        // this is the default short-form behavior
        let file_names = utils::collect_file_names(&path, &params)?;
        let max_name_length = utils::calculate_max_name_length(&file_names);
        let terminal_width =
            term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        let num_columns = terminal_width / max_name_length;

        let mut table = utils::create_table(2);
        utils::add_files_to_table(&mut table, &file_names, num_columns);

        table.printstd();
    }

    Ok(())
}
