use chrono::{DateTime, Local};
use clap::Parser;
use glob::glob;
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::time::SystemTime;

mod cli;
mod utils;

use utils::icons::Icon;

struct Params {
    show_all: bool,
    append_slash: bool,
    dirs_first: bool,
    almost_all: bool,
    long_format: bool,
    human_readable: bool,
    no_icons: bool,
    fuzzy_time: bool,
}

struct FileInfo {
    file_type: String,
    mode: String,
    nlink: u64,
    user: String,
    group: String,
    size: u64,
    mtime: SystemTime,
    item_icon: Option<Icon>,
    display_name: String,
    full_path: PathBuf,
}

fn main() {
    let args = cli::Flags::parse();
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    let params = Params {
        show_all: args.show_all,
        append_slash: args.slash,
        dirs_first: args.dirs_first,
        almost_all: args.almost_all,
        long_format: args.long,
        human_readable: args.human_readable,
        no_icons: args.no_icons,
        fuzzy_time: args.fuzzy_time,
    };

    let patterns = if args.paths.is_empty() {
        vec![String::from(".")]
    } else {
        args.paths
    };

    if let Err(e) = run_multi(&patterns, &params) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run_multi(patterns: &[String], params: &Params) -> io::Result<()> {
    let mut all_file_info = Vec::new();

    for pattern in patterns {
        match glob(pattern) {
            Ok(entries) => {
                let paths: Vec<PathBuf> =
                    entries.filter_map(Result::ok).collect();
                if paths.is_empty() {
                    eprintln!("lsp: {}: No such file or directory", pattern);
                } else {
                    for path in paths {
                        let file_info = collect_file_info(&path, params)?;
                        all_file_info.extend(file_info);
                    }
                }
            }
            Err(e) => eprintln!("Failed to read glob pattern: {}", e),
        }
    }

    if params.long_format {
        display_long_format(&all_file_info, params)
    } else {
        display_short_format(&all_file_info, params)
    }
}

// fn handle_error(path: &str, e: io::Error) {
//     let error_message = match e.kind() {
//         io::ErrorKind::PermissionDenied => "Permission denied",
//         io::ErrorKind::NotFound => "No such file or directory",
//         _ => &e.to_string(),
//     };
//     eprintln!("lsp: {}: {}", path, error_message);
// }

fn collect_file_info(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<FileInfo>> {
    let mut file_info = Vec::new();
    let metadata = fs::symlink_metadata(path)?;

    if metadata.is_dir() {
        let file_names = utils::file::collect_file_names(path, params)?;
        for file_name in file_names {
            let full_path = path.join(&file_name);
            if let Ok(info) = create_file_info(&full_path, params) {
                file_info.push(info);
            }
        }
    } else if let Ok(info) = create_file_info(path, params) {
        file_info.push(info);
    }
    Ok(file_info)
}

fn create_file_info(path: &Path, params: &Params) -> io::Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    let item_icon = if params.no_icons {
        None
    } else {
        Some(utils::icons::get_item_icon(
            &metadata,
            &path.to_string_lossy(),
        ))
    };
    let (file_type, mode, nlink, size, mtime, user, group, executable) =
        utils::file::get_file_details(&metadata);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    if params.append_slash && metadata.is_dir() {
        file_name.push('/');
    }

    let display_name = if metadata.is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                let target_path = if target.is_relative() {
                    path.parent().unwrap_or(Path::new("")).join(target)
                } else {
                    target
                };
                if target_path.exists() {
                    format!(
                        "{color_cyan}{} -> {}",
                        file_name,
                        target_path.display()
                    )
                } else {
                    format!(
                        "{color_cyan}{} -> {} {color_red}[Broken Link]",
                        file_name,
                        target_path.display()
                    )
                }
            }
            Err(_) => format!("{color_red}{} -> (unreadable)", file_name),
        }
    } else if metadata.is_dir() {
        format!("{color_blue}{}", file_name)
    } else if executable {
        format!("{style_bold}{color_green}{}", file_name)
    } else {
        file_name.clone()
    };

    Ok(FileInfo {
        file_type,
        mode,
        nlink,
        user,
        group,
        size,
        mtime,
        item_icon,
        display_name,
        full_path: path.to_path_buf(),
    })
}

fn display_long_format(
    file_info: &[FileInfo],
    params: &Params,
) -> io::Result<()> {
    let mut table = utils::table::create_table(0);

    for info in file_info {
        let display_time = if params.fuzzy_time {
            utils::fuzzy_time(info.mtime).to_string()
        } else {
            let datetime: DateTime<Local> = DateTime::from(info.mtime);
            datetime.format("%c").to_string()
        };

        let (display_size, units) =
            utils::format::show_size(info.size, params.human_readable);

        let mut row_cells = Vec::with_capacity(9);

        row_cells
            .push(Cell::new(&format!("{}{} ", info.file_type, info.mode)));
        row_cells.push(Cell::new(&info.nlink.to_string()));
        row_cells.push(Cell::new(&format!(" {color_cyan}{}", info.user)));
        row_cells.push(Cell::new(&format!("{color_green}{} ", info.group)));
        row_cells.push(Cell::new(&display_size).style_spec("r"));

        if !units.is_empty() {
            row_cells.push(Cell::new(units));
        }

        row_cells.push(
            Cell::new(&format!(" {color_yellow}{} ", display_time))
                .style_spec("r"),
        );

        if let Some(icon) = &info.item_icon {
            row_cells.push(Cell::new(&format!("{} ", icon)));
        }

        row_cells.push(Cell::new(&info.display_name));

        table.add_row(Row::new(row_cells));
    }

    table.printstd();
    Ok(())
}

fn display_short_format(
    file_info: &[FileInfo],
    _params: &Params,
) -> io::Result<()> {
    let max_name_length = file_info
        .iter()
        .map(|info| info.display_name.len())
        .max()
        .unwrap_or(0)
        + 2; // Adding space between columns

    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let num_columns = terminal_width / max_name_length;

    let mut table = utils::table::create_table(2);

    for chunk in file_info.chunks(num_columns) {
        let mut row = Row::empty();
        for info in chunk {
            let display_name = info.display_name.clone();

            let mut cell_content = String::new();
            if let Some(icon) = &info.item_icon {
                cell_content.push_str(&format!("{} ", icon));
            }
            cell_content.push_str(&display_name);

            row.add_cell(Cell::new(&cell_content));
        }
        table.add_row(row);
    }

    table.printstd();
    Ok(())
}
