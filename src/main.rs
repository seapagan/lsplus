#[macro_use]
extern crate prettytable;

mod utils;

use clap::{Arg, Command};
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let matches = Command::new("rls")
        .version("0.0.1")
        .author("Grant Ramsay <seapagan@gmail.com>")
        .about("A replacement for the ls command written in Rust.")
        .arg(
            Arg::new("long")
                .short('l')
                .long("long")
                .help("Display detailed information")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(Arg::new("path").default_value(".").help("The path to list"))
        .arg(
            Arg::new("slash")
                .short('p')
                .long("slash-dirs")
                .help("Append a slash to directories")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    let long_format = matches.get_flag("long");
    let append_slash = matches.get_flag("slash");

    if long_format {
        let mut table = utils::create_table(1);
        let file_names = utils::collect_file_names(path, append_slash, true)?;

        for file_name in file_names {
            let metadata =
                fs::symlink_metadata(format!("{}/{}", path, file_name))?;
            let item_icon = utils::get_item_icon(&metadata);
            let (_file_type, mode, nlink, size, mtime, user, group) =
                utils::get_file_details(&metadata);

            table.add_row(row![
                mode, nlink, user, group, size, mtime, item_icon, file_name,
            ]);
        }
        table.printstd();
    } else {
        // this is the default short-form behavior
        let file_names = utils::collect_file_names(path, append_slash, false)?;
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
