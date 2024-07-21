#[macro_use]
extern crate prettytable;

use clap::{ArgAction, Parser};
mod utils;
use inline_colorization::*;
use std::fs;
use std::io;
use std::path::PathBuf;

// Set up the CLI arguments
#[derive(Parser)]
#[command(
    name = "lsplus",
    version = "0.1.0",
    author = "Grant Ramsay <seapagan@gmail.com>",
    about = "A replacement for the 'ls' command written in Rust."
)]
struct Cli {
    #[arg(short='l', long="long", action = ArgAction::SetTrue, help = "Display detailed information")]
    long: bool,

    #[arg(default_value = ".", help = "The path to list")]
    path: String,

    #[arg(short = 'p', long = "slash-dirs", action = ArgAction::SetTrue, help = "Append a slash to directories")]
    slash: bool,

    #[arg(short = 'd', long = "dirs-first", action = ArgAction::SetTrue, help = "Sort directories first")]
    dirs_first: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    // read in the command line arguments
    let path = args.path;
    let long_format = args.long;
    let append_slash = args.slash;
    let dirs_first = args.dirs_first;

    // different behavior for long format or short format
    if long_format {
        let mut table = utils::create_table(1);
        let file_names =
            utils::collect_file_names(&path, append_slash, dirs_first, true)?;

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

            table.add_row(row![
                format!("{}{}", file_type, mode),
                nlink,
                format!("{color_cyan}{}", user),
                format!("{color_green}{}", group),
                size,
                format!("{color_yellow}{}", mtime),
                item_icon,
                // utils::get_filename_from_path(&display_name),
                display_name,
            ]);
        }
        table.printstd();
    } else {
        // this is the default short-form behavior
        let file_names =
            utils::collect_file_names(&path, append_slash, dirs_first, false)?;
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
