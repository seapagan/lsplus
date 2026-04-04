use chrono::{DateTime, Local};
use colored_text::{ColorMode, Colorize, ColorizeConfig, StyledText};
use prettytable::{Cell, Row, Table};
use std::io::{self, IsTerminal, Write};

use strip_ansi_escapes::strip_str;
use terminal_size::{Height, Width, terminal_size};
use unicode_width::UnicodeWidthStr;

use crate::Params;
use crate::structs::{FileInfo, NameStyle};
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
    if short_output_uses_colored_cells() {
        print_short_lines(&render_short_format_lines(
            file_info,
            terminal_width,
        ))
    } else {
        print_table(&build_short_format_table(file_info, terminal_width))
    }
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

pub(crate) fn render_short_format_lines(
    file_info: &[FileInfo],
    terminal_width: usize,
) -> Vec<String> {
    let rows = short_rows(file_info, terminal_width);
    let column_widths = short_column_widths(&rows);

    rows.iter()
        .map(|row| render_short_row(row, &column_widths))
        .collect()
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
    visible_text_width(&plain_short_cell_content(info)) + SHORT_CELL_PADDING
}

fn short_cell_content(info: &FileInfo) -> String {
    let (prefix, _) = short_cell_parts(info);
    let mut cell_content = prefix;
    cell_content.push_str(&check_display_name(info));
    cell_content
}

fn plain_short_cell_content(info: &FileInfo) -> String {
    let (prefix, _) = short_cell_parts(info);
    let display_name = check_display_name(info);
    let mut cell_content = prefix;
    if display_name == info.display_name {
        cell_content.push_str(&info.short_name);
    } else {
        cell_content.push_str(&strip_str(&display_name));
    }
    cell_content
}

fn short_cell_parts(info: &FileInfo) -> (String, String) {
    let prefix = info
        .item_icon
        .as_ref()
        .map(|icon| format!("{} ", icon))
        .unwrap_or_default();
    let display_name = check_display_name(info);
    let name = if display_name == info.display_name {
        info.short_name.clone()
    } else {
        strip_str(&display_name)
    };
    (prefix, name)
}

fn short_column_widths(rows: &[&[FileInfo]]) -> Vec<usize> {
    let mut widths =
        vec![0; rows.iter().map(|row| row.len()).max().unwrap_or(0)];

    for row in rows {
        for (index, info) in row.iter().enumerate() {
            widths[index] = widths[index]
                .max(visible_text_width(&plain_short_cell_content(info)));
        }
    }

    widths
}

fn render_short_row(row: &[FileInfo], column_widths: &[usize]) -> String {
    let mut line = String::from(" ");

    for (index, info) in row.iter().enumerate() {
        let (prefix, name) = short_cell_parts(info);
        line.push_str(&prefix);
        line.push_str(&style_short_segment(
            info,
            padded_short_name(&prefix, &name, column_widths[index]),
        ));

        if index + 1 < row.len() {
            line.push(' ');
        }
    }

    line
}

fn padded_short_name(prefix: &str, name: &str, column_width: usize) -> String {
    let full_width = visible_text_width(&format!("{prefix}{name}"));
    let right_padding =
        column_width.saturating_sub(full_width) + SHORT_CELL_PADDING;
    let mut padded = String::from(name);
    padded.push_str(&" ".repeat(right_padding));
    padded
}

fn style_short_segment(info: &FileInfo, text: String) -> String {
    let styled = match info.name_style {
        NameStyle::Plain => StyledText::plain(text),
        NameStyle::Directory => text.blue(),
        NameStyle::Symlink => text.cyan(),
        NameStyle::Executable => text.green().bold(),
    };

    if info.dimmed {
        styled.dim().to_string()
    } else {
        styled.to_string()
    }
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

fn print_short_lines(lines: &[String]) -> io::Result<()> {
    let mut stdout = io::stdout();
    for line in lines {
        writeln!(stdout, "{line}")?;
    }
    stdout.flush()
}

fn short_output_uses_colored_cells() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }

    match ColorizeConfig::color_mode() {
        ColorMode::Never => false,
        ColorMode::Always => true,
        ColorMode::Auto => std::io::stdout().is_terminal(),
    }
}
