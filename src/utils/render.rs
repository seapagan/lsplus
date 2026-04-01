use chrono::{DateTime, Local};
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::io;

use strip_ansi_escapes::strip_str;
use terminal_size::{Width, terminal_size};
use unicode_width::UnicodeWidthStr;

use crate::Params;
use crate::structs::FileInfo;
use crate::utils;
use crate::utils::file::check_display_name;

const SHORT_CELL_PADDING: usize = 2;

pub fn display_long_format(
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

        let display_name = check_display_name(info);
        row_cells.push(Cell::new(&display_name));
        table.add_row(Row::new(row_cells));
    }

    table.printstd();
    Ok(())
}

pub fn display_short_format(file_info: &[FileInfo]) -> io::Result<()> {
    let terminal_width = terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .unwrap_or(80);
    let num_columns = short_column_count(file_info, terminal_width);

    let mut table = utils::table::create_table(2);

    for chunk in file_info.chunks(num_columns) {
        let mut row = Row::empty();
        for info in chunk {
            row.add_cell(Cell::new(&short_cell_content(info)));
        }
        table.add_row(row);
    }

    table.printstd();
    Ok(())
}

fn short_column_count(file_info: &[FileInfo], terminal_width: usize) -> usize {
    let max_cell_width = file_info
        .iter()
        .map(short_cell_width)
        .max()
        .unwrap_or(SHORT_CELL_PADDING);

    (terminal_width / max_cell_width).max(1)
}

fn short_cell_width(info: &FileInfo) -> usize {
    visible_text_width(&short_cell_content(info)) + SHORT_CELL_PADDING
}

fn short_cell_content(info: &FileInfo) -> String {
    let display_name = check_display_name(info);
    let mut cell_content = String::new();
    if let Some(icon) = &info.item_icon {
        cell_content.push_str(&format!("{} ", icon));
    }
    cell_content.push_str(&display_name);
    cell_content
}

fn visible_text_width(text: &str) -> usize {
    let stripped = strip_str(text);
    UnicodeWidthStr::width(stripped.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::icons::Icon;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn test_file_info(
        display_name: &str,
        item_icon: Option<Icon>,
    ) -> FileInfo {
        FileInfo {
            file_type: String::from("-"),
            mode: String::from("rw-r--r--"),
            nlink: 1,
            user: String::from("user"),
            group: String::from("group"),
            size: 0,
            mtime: SystemTime::now(),
            item_icon,
            display_name: display_name.to_string(),
            full_path: PathBuf::from(display_name),
        }
    }

    #[test]
    fn test_visible_text_width_strips_ansi_and_uses_display_width() {
        let styled = format!("{color_red}界{color_reset}");
        assert_eq!(visible_text_width(&styled), 2);
    }

    #[test]
    fn test_short_cell_width_includes_icon_width() {
        let plain = test_file_info("example.rs", None);
        let with_icon = test_file_info("example.rs", Some(Icon::RustFile));

        assert!(short_cell_width(&with_icon) > short_cell_width(&plain));
    }

    #[test]
    fn test_short_column_count_never_returns_zero() {
        let files = [test_file_info("very-long-filename.txt", None)];

        assert_eq!(short_column_count(&files, 1), 1);
    }

    #[test]
    fn test_short_cell_content_includes_icon_and_display_name() {
        let file_info = test_file_info("example.rs", Some(Icon::RustFile));
        let content = short_cell_content(&file_info);

        assert!(content.contains("example.rs"));
        assert!(content.contains(&Icon::RustFile.to_string()));
    }
}
