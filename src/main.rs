#[macro_use]
extern crate prettytable;

mod utils;

use clap::{Arg, Command};
use prettytable::format::FormatBuilder;
use prettytable::Table;
use std::fs;
use std::io;
use term_size;

fn main() -> io::Result<()> {
    let matches = Command::new("ls_replacement")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A replacement for the ls command")
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

            let file_name = utils::get_file_name_with_slash(&metadata, &file_name, append_slash);
            let item_icon = utils::get_item_icon(&metadata);
            let (file_type, mode, nlink, size, mtime) = utils::get_file_details(&metadata);

            table.add_row(row![
                file_type, mode, nlink, size, mtime, item_icon, file_name,
            ]);
        }
        table.printstd();
    } else {
        //     let mut file_names = Vec::new();
        //     for entry in entries {
        //         let entry = entry?;
        //         let file_name = entry.file_name().into_string().unwrap();
        //         let metadata = entry.metadata()?;
        //         let file_name = utils::get_file_name_with_slash(&metadata, &file_name, append_slash);
        //
        //         file_names.push(file_name);
        //     }
        //     println!("{}", file_names.join(" "));
        // }
        let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);

        // Collect file names and their lengths
        let mut file_names = Vec::new();
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_name = entry.file_name().into_string().unwrap();
            let file_name = utils::get_file_name_with_slash(&metadata, &file_name, append_slash);
            file_names.push(file_name);
        }

        // Calculate maximum filename length
        let max_name_length = file_names.iter().map(|name| name.len()).max().unwrap_or(0) + 2; // Adding space between columns
        let num_columns = terminal_width / max_name_length;

        // Create a new table with no borders or padding
        let mut table = Table::new();
        let format = FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(0, 2)
            .build();
        table.set_format(format);

        // Add filenames to the table
        for chunk in file_names.chunks(num_columns) {
            let mut row = prettytable::Row::empty();
            for cell in chunk.iter() {
                row.add_cell(prettytable::Cell::new(cell));
            }
            table.add_row(row);
        }

        table.printstd();
    }

    Ok(())
}
