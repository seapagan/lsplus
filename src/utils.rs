use chrono::{DateTime, Local};
use nix::unistd::{Group, User};
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
        "\u{f07c}".to_string()
    } else if metadata.is_symlink() {
        "\u{f1177}".to_string()
    } else {
        "".to_string()
    }
}

pub fn get_file_details(
    metadata: &fs::Metadata,
) -> (String, String, u64, u64, String, String, String) {
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
    let rwx_mode = mode_to_rwx(mode);

    let nlink = metadata.nlink();
    let size = metadata.size();

    let user = get_username(metadata.uid());
    let group = get_groupname(metadata.gid());

    let modified_time = metadata.modified().unwrap();
    let datetime: DateTime<Local> = DateTime::from(modified_time);
    let mtime = datetime.format("%c").to_string();

    (file_type, rwx_mode, nlink, size, mtime, user, group)
}

pub fn calculate_max_name_length(file_names: &[String]) -> usize {
    file_names.iter().map(|name| name.len()).max().unwrap_or(0) + 2 // Adding space between columns
}

pub fn collect_file_names(
    path: &String,
    append_slash: bool,
    show_dotdot: bool,
) -> io::Result<Vec<String>> {
    let entries = fs::read_dir(path)?;
    let mut file_names = Vec::new();

    if show_dotdot {
        file_names = vec![".".to_string(), "..".to_string()]; // Initialize with . and ..
    }
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::symlink_metadata(&path)?;

        let file_name = entry.file_name().into_string().unwrap();
        let file_name =
            get_file_name_with_slash(&metadata, &file_name, append_slash);
        file_names.push(file_name);
    }
    Ok(file_names)
}

pub fn create_table(padding: usize) -> Table {
    let format = FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(0, padding)
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

pub fn mode_to_rwx(mode: u32) -> String {
    let mut rwx = String::new();
    let perms = [
        (mode & 0o400, 'r'),
        (mode & 0o200, 'w'),
        (mode & 0o100, 'x'), // Owner
        (mode & 0o040, 'r'),
        (mode & 0o020, 'w'),
        (mode & 0o010, 'x'), // Group
        (mode & 0o004, 'r'),
        (mode & 0o002, 'w'),
        (mode & 0o001, 'x'), // Others
    ];

    for (bit, chr) in perms.iter() {
        if *bit != 0 {
            rwx.push(*chr);
        } else {
            rwx.push('-');
        }
    }

    rwx
}

pub fn get_username(uid: u32) -> String {
    match User::from_uid(uid.into()) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

pub fn get_groupname(gid: u32) -> String {
    match Group::from_gid(gid.into()) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}
