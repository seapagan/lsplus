use chrono::{DateTime, Local};
use nix::unistd::{Group, User};
use prettytable::{format::FormatBuilder, Cell, Row, Table};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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
    show_all: bool,
    append_slash: bool,
    dirs_first: bool,
    almost_all: bool,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let path_metadata = fs::symlink_metadata(path)?;

    if !path_metadata.is_dir() {
        // If it's a file or symlink, add it directly to the file_names vector
        let file_name = PathBuf::from(path)
            // .file_name()
            // .unwrap()
            .to_string_lossy()
            .into_owned();
        file_names.push(file_name);
    } else {
        // If it's a directory, read its entries
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if show_all || almost_all {
                    true
                } else {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.'))
                        .unwrap_or(false)
                }
            })
            .collect();

        // Sort entries alphabetically, ignoring leading dots
        entries.sort_by(|a, b| {
            let a_name = a
                .file_name()
                .into_string()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            let b_name = b
                .file_name()
                .into_string()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            a_name.cmp(&b_name)
        });

        // Separate directories and files if dirs_first is true
        if dirs_first {
            let (dirs, files): (Vec<_>, Vec<_>) =
                entries.into_iter().partition(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                });

            entries = dirs.into_iter().chain(files).collect();
        }

        if !almost_all && show_all {
            if append_slash {
                file_names = vec!["./".to_string(), "../".to_string()];
            } else {
                file_names = vec![".".to_string(), "..".to_string()];
            }
        }

        for entry in entries {
            let metadata = fs::symlink_metadata(entry.path())?;
            let mut file_name = entry.file_name().into_string().unwrap();
            file_name =
                get_file_name_with_slash(&metadata, &file_name, append_slash);
            file_names.push(file_name);
        }
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

#[allow(dead_code)]
pub fn get_filename_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

pub fn human_readable_format(size: u64) -> (f64, &'static str) {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    (size, UNITS[unit_index])
}

pub fn show_size(size: u64, human_readable: bool) -> (String, &'static str) {
    if human_readable {
        let (size, unit) = human_readable_format(size);
        if size.fract() == 0.0 {
            (format!("{:.0}", size), unit)
        } else {
            (format!("{:.1}", size), unit)
        }
    } else {
        (size.to_string(), "")
    }
}
