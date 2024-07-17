use chrono::{DateTime, Local};
use prettytable::{format::FormatBuilder, Cell, Row, Table};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

pub fn get_file_name_with_slash(
    metadata: &fs::Metadata,
    file_name: &str,
    append_slash: bool,
) -> String {
    if metadata.is_dir() && append_slash {
        format!("{}/", file_name)
    } else {
        file_name.to_string()
    }
}

pub fn get_item_icon(metadata: &fs::Metadata) -> String {
    if metadata.is_dir() {
        "d".to_string() // Placeholder icon
    } else {
        "".to_string()
    }
}

pub fn get_file_details(
    metadata: &fs::Metadata,
) -> (String, String, u64, u64, String) {
    let file_type = if metadata.is_dir() {
        "d"
    } else if metadata.is_file() {
        "-"
    } else if metadata.is_symlink() {
        "l"
    } else {
        "?"
    }
    .to_string();

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    let nlink = metadata.nlink();
    let size = metadata.size();

    let modified_time = metadata.modified().unwrap();
    let datetime: DateTime<Local> = DateTime::from(modified_time);
    let mtime = datetime.format("%c").to_string();

    (file_type, format!("{:o}", mode & 0o777), nlink, size, mtime)
}

pub fn calculate_max_name_length(file_names: &[String]) -> usize {
    file_names.iter().map(|name| name.len()).max().unwrap_or(0) + 2 // Adding space between columns
}

pub fn collect_file_names(
    entries: fs::ReadDir,
    append_slash: bool,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_name = entry.file_name().into_string().unwrap();
        let file_name =
            get_file_name_with_slash(&metadata, &file_name, append_slash);
        file_names.push(file_name);
    }
    Ok(file_names)
}

pub fn create_table() -> Table {
    let format = FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(0, 2)
        .build();
    let mut table = Table::new();
    table.set_format(format);
    table
}

pub fn add_files_to_table(
    table: &mut Table,
    file_names: &[String],
    num_columns: usize,
) {
    for chunk in file_names.chunks(num_columns) {
        let mut row = Row::empty();
        for cell in chunk.iter() {
            row.add_cell(Cell::new(cell));
        }
        table.add_row(row);
    }
}
