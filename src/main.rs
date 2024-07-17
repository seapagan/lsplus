#[macro_use]
extern crate prettytable;

mod utils;

use clap::{Arg, Command};
use prettytable::format::FormatBuilder;
use prettytable::Table;
use std::fs;
use std::io;

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
        // Print filenames in a single line separated by spaces
        let mut file_names = Vec::new();
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            let metadata = entry.metadata()?;
            let file_name = utils::get_file_name_with_slash(&metadata, &file_name, append_slash);

            file_names.push(file_name);
        }
        println!("{}", file_names.join(" "));
    }

    Ok(())
}
