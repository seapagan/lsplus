use chrono::{DateTime, Local};
use std::fs;
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

pub fn get_file_details(metadata: &fs::Metadata) -> (String, String, u64, u64, String) {
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
