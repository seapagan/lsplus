#[macro_use]
extern crate prettytable;

mod utils;

use clap::{Arg, Command};
use prettytable::format::FormatBuilder;
use prettytable::Table;
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
                .short('s')
                .help("Append a slash to directories")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    let long_format = matches.get_flag("long");
    let append_slash = matches.get_flag("slash");

    let entries = fs::read_dir(path)?;

    let mut table = Table::new();
    let format = FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(1, 1)
        .build();

    table.set_format(format);

    if long_format {
        table.set_titles(row![
            "Type",
            "Mode",
            "Links",
            "Size",
            "Modified Time",
            "",
            "Name"
        ]);
    }

    if long_format {
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_name = entry.file_name().into_string().unwrap();

            let file_name = utils::get_file_name_with_slash(
                &metadata,
                &file_name,
                append_slash,
            );
            let item_icon = utils::get_item_icon(&metadata);
            let (file_type, mode, nlink, size, mtime) =
                utils::get_file_details(&metadata);

            table.add_row(row![
                file_type, mode, nlink, size, mtime, item_icon, file_name,
            ]);
        }
        table.printstd();
    } else {
        // this is the default short-form behavior
        let file_names = utils::collect_file_names(entries, append_slash)?;
        let max_name_length = utils::calculate_max_name_length(&file_names);
        let terminal_width =
            term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        let num_columns = terminal_width / max_name_length;

        let mut table = utils::create_table();
        utils::add_files_to_table(&mut table, &file_names, num_columns);

        table.printstd();
    }

    Ok(())
}
