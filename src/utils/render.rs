use chrono::{DateTime, Local};
use colored_text::Colorize;
use prettytable::{Cell, Row, Table};
use std::io::{self, Write};

use strip_ansi_escapes::strip_str;
use terminal_size::{Height, Width, terminal_size};
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
    print_table(&build_long_format_table(file_info, params))
}

pub(crate) fn build_long_format_table(
    file_info: &[FileInfo],
    params: &Params,
) -> Table {
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
        row_cells.push(Cell::new(&format!(" {}", info.user.cyan())));
        row_cells.push(Cell::new(&format!("{} ", info.group.green())));
        row_cells.push(Cell::new(&display_size).style_spec("r"));

        if !units.is_empty() {
            row_cells.push(Cell::new(units));
        }

        row_cells.push(
            Cell::new(&format!(" {} ", display_time.yellow())).style_spec("r"),
        );

        if let Some(icon) = &info.item_icon {
            row_cells.push(Cell::new(&format!("{} ", icon)));
        }

        let display_name = check_display_name(info);
        row_cells.push(Cell::new(&display_name));
        table.add_row(Row::new(row_cells));
    }

    table
}

pub fn display_short_format(file_info: &[FileInfo]) -> io::Result<()> {
    let terminal_width = terminal_width_or_default(terminal_size());
    print_table(&build_short_format_table(file_info, terminal_width))
}

pub(crate) fn build_short_format_table(
    file_info: &[FileInfo],
    terminal_width: usize,
) -> Table {
    let rows = short_rows(file_info, terminal_width);
    let mut table = utils::table::create_table(2);

    for chunk in rows {
        let mut row = Row::empty();
        for info in chunk {
            row.add_cell(Cell::new(&short_cell_content(info)));
        }
        table.add_row(row);
    }

    table
}

pub(crate) fn terminal_width_or_default(
    size: Option<(Width, Height)>,
) -> usize {
    size.map(|(Width(width), _)| usize::from(width))
        .unwrap_or(80)
}

fn short_rows(
    file_info: &[FileInfo],
    terminal_width: usize,
) -> Vec<&[FileInfo]> {
    let num_columns = short_column_count(file_info, terminal_width);
    file_info.chunks(num_columns).collect()
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

fn print_table(table: &Table) -> io::Result<()> {
    let mut stdout = io::stdout();
    table.print(&mut stdout)?;
    stdout.flush()
}
