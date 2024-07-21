use clap::{Arg, ArgAction, Parser};
mod utils;
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::exit;

fn version_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    // Provide default values if fields are empty
    let authors = if authors.is_empty() {
        "Unknown"
    } else {
        authors
    };
    let description = if description.is_empty() {
        "No description provided"
    } else {
        description
    };

    format!(
        "lsplus v{}\n\
        \n{}\n\
        \nReleased under the MIT license by {}\n",
        version, description, authors
    )
}

// Set up the CLI arguments
#[derive(Parser)]
#[command(
    name = "lsplus",
    author = env!("CARGO_PKG_AUTHORS"),
    about =env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_flag = true,
    arg(
        Arg::new("help")
            .long("help")
            .action(ArgAction::Help)
            .help("Print help information")
    )
)]
struct Cli {
    #[arg(short ='a', long = "all", action = ArgAction::SetTrue, help = "Do not ignore entries starting with .")]
    show_all: bool,

    #[arg(short ='A', long = "almost-all", action = ArgAction::SetTrue, help = "Do not list implied . and ..")]
    almost_all: bool,

    #[arg(short='l', long="long", action = ArgAction::SetTrue, help = "Display detailed information")]
    long: bool,

    #[arg(short='h', long="human-readable", action = ArgAction::SetTrue, help = "with -l, print sizes like 1K 234M 2G etc.")]
    human_readable: bool,

    #[arg(default_value = ".", help = "The path to list")]
    path: String,

    #[arg(short = 'p', long = "slash-dirs", action = ArgAction::SetTrue, help = "Append a slash to directories")]
    slash: bool,

    #[arg(short = 'D', long = "sort-dirs", action = ArgAction::SetTrue, help = "Sort directories first")]
    dirs_first: bool,
    #[arg(
        long = "version",
        short = 'V',
        action = ArgAction::SetTrue,
        help = "Print version information and exit",
        global = true
    )]
    version: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    if args.version {
        println!("{}", version_info());
        exit(0);
    }
    // read in the command line arguments
    let path = args.path;
    let long_format = args.long;
    let append_slash = args.slash;
    let dirs_first = args.dirs_first;
    let show_all = args.show_all;
    let almost_all = args.almost_all;
    let human_readable = args.human_readable;

    // different behavior for long format or short format
    if long_format {
        let mut table = utils::create_table(0);
        let file_names = utils::collect_file_names(
            &path,
            show_all,
            append_slash,
            dirs_first,
            almost_all,
        )?;

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

            let (display_size, units) = utils::show_size(size, human_readable);

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
        let file_names = utils::collect_file_names(
            &path,
            show_all,
            append_slash,
            dirs_first,
            almost_all,
        )?;
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
