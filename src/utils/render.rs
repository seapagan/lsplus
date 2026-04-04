use chrono::{DateTime, Local};
use colored_text::{Colorize, StyledText};
use prettytable::{Cell, Row, Table};
use std::io::{self, Write};

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
    print_short_lines(&render_short_format_lines(file_info, terminal_width))
}

pub(crate) fn render_short_format_lines(
    file_info: &[FileInfo],
    terminal_width: usize,
) -> Vec<String> {
    let render_items = short_render_items(file_info);
    let rows = short_rows(&render_items, terminal_width);
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

fn short_rows<'a>(
    render_items: &'a [ShortRenderItem<'a>],
    terminal_width: usize,
) -> Vec<&'a [ShortRenderItem<'a>]> {
    let num_columns = short_column_count(render_items, terminal_width);
    render_items.chunks(num_columns).collect()
}

fn short_column_count(
    render_items: &[ShortRenderItem<'_>],
    terminal_width: usize,
) -> usize {
    let max_cell_width = render_items
        .iter()
        .map(short_cell_width)
        .max()
        .unwrap_or(SHORT_CELL_PADDING);

    (terminal_width / max_cell_width).max(1)
}

fn short_cell_width(item: &ShortRenderItem<'_>) -> usize {
    item.plain_width + SHORT_CELL_PADDING
}

struct ShortRenderItem<'a> {
    info: &'a FileInfo,
    prefix: String,
    name: String,
    plain_width: usize,
}

fn short_render_items(file_info: &[FileInfo]) -> Vec<ShortRenderItem<'_>> {
    file_info.iter().map(short_render_item).collect()
}

fn short_render_item(info: &FileInfo) -> ShortRenderItem<'_> {
    let display_name = check_display_name(info);
    let (prefix, name) = short_cell_parts(info, &display_name);
    let plain_width = visible_text_width(&format!("{prefix}{name}"));

    ShortRenderItem {
        info,
        prefix,
        name,
        plain_width,
    }
}

fn short_cell_parts(info: &FileInfo, display_name: &str) -> (String, String) {
    let prefix = info
        .item_icon
        .as_ref()
        .map(|icon| format!("{} ", icon))
        .unwrap_or_default();
    let name = if display_name == info.display_name.as_str() {
        info.short_name.clone()
    } else {
        strip_str(display_name)
    };
    (prefix, name)
}

fn short_column_widths(rows: &[&[ShortRenderItem<'_>]]) -> Vec<usize> {
    let mut widths =
        vec![0; rows.iter().map(|row| row.len()).max().unwrap_or(0)];

    for row in rows {
        for (index, item) in row.iter().enumerate() {
            widths[index] = widths[index].max(item.plain_width);
        }
    }

    widths
}

fn render_short_row(
    row: &[ShortRenderItem<'_>],
    column_widths: &[usize],
) -> String {
    let mut line = String::from(" ");

    for (index, item) in row.iter().enumerate() {
        let is_last_column = index + 1 == row.len();
        line.push_str(&item.prefix);
        line.push_str(&style_short_segment(
            item.info,
            padded_short_name(
                &item.name,
                column_widths[index],
                item.plain_width,
                is_last_column,
            ),
        ));

        if !is_last_column {
            line.push(' ');
        }
    }

    line
}

fn padded_short_name(
    name: &str,
    column_width: usize,
    plain_width: usize,
    is_last_column: bool,
) -> String {
    let right_padding = if is_last_column {
        SHORT_CELL_PADDING
    } else {
        column_width.saturating_sub(plain_width) + SHORT_CELL_PADDING
    };
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
