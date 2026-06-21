use chrono::{DateTime, Local};
use colored_text::{Colorize, StyledText};
use prettytable::{Cell, Row, Table};
use std::fmt::Write as FmtWrite;
use std::io::{self, Write as IoWrite};
use std::time::{Duration, SystemTime};

use strip_ansi_escapes::strip_str;
use terminal_size::{Height, Width, terminal_size};
use unicode_width::UnicodeWidthStr;

use crate::Params;
use crate::structs::{FileInfo, NameStyle};
use crate::utils;
use crate::utils::color::{LongFormatColorLevel, long_format_color_level};
use crate::utils::file::check_display_name;

const SHORT_CELL_PADDING: usize = 2;
const LARGE_SIZE_BYTES: u64 = 1024 * 1024;
const HUGE_SIZE_BYTES: u64 = 1024 * 1024 * 1024;
const DAY: Duration = Duration::from_secs(24 * 60 * 60);
const WEEK: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MONTH: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const YEAR: Duration = Duration::from_secs(365 * 24 * 60 * 60);

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
    let color_level = long_format_color_level(params);

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

        row_cells.push(Cell::new(&format!(
            "{} ",
            long_permission_text(info, params)
        )));
        row_cells.push(Cell::new(&info.nlink.to_string()));
        row_cells.push(Cell::new(&format!(" {}", info.user.cyan())));
        row_cells.push(Cell::new(&format!("{} ", info.group.green())));
        row_cells.push(size_cell(
            &display_size,
            info.size,
            params,
            color_level,
            "r",
        ));

        if !units.is_empty() {
            row_cells.push(size_cell(
                units,
                info.size,
                params,
                color_level,
                "",
            ));
        }

        row_cells.push(
            Cell::new(&format!(
                " {} ",
                long_time_text(&display_time, info.mtime, params, color_level)
            ))
            .style_spec("r"),
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

fn long_permission_text(info: &FileInfo, params: &Params) -> String {
    if !params.permission_colors {
        return format!("{}{}", info.file_type, info.mode);
    }

    let mut output =
        String::with_capacity(info.file_type.len() + info.mode.len());
    for value in info.file_type.chars() {
        write_file_type_char(&mut output, value);
    }
    for value in info.mode.chars() {
        write_permission_char(&mut output, value);
    }

    output
}

fn write_file_type_char(output: &mut String, value: char) {
    match value {
        'd' => write!(output, "{}", value.blue()).unwrap(),
        'l' => write!(output, "{}", value.cyan()).unwrap(),
        '-' => write!(output, "{}", value.dim()).unwrap(),
        _ => output.push(value),
    }
}

fn write_permission_char(output: &mut String, value: char) {
    match value {
        'r' => write!(output, "{}", value.green()).unwrap(),
        'w' => write!(output, "{}", value.yellow()).unwrap(),
        'x' => write!(output, "{}", value.red().bold()).unwrap(),
        '-' => write!(output, "{}", value.dim()).unwrap(),
        _ => output.push(value),
    }
}

fn size_cell(
    text: &str,
    size: u64,
    params: &Params,
    color_level: LongFormatColorLevel,
    base: &str,
) -> Cell {
    Cell::new(text).style_spec(size_style_spec_for_color_level(
        size,
        params,
        color_level,
        base,
    ))
}

pub(crate) fn size_style_spec_for_color_level(
    size: u64,
    params: &Params,
    color_level: LongFormatColorLevel,
    base: &str,
) -> &'static str {
    match (params.size_colors && color_level.is_enabled(), size, base) {
        (true, HUGE_SIZE_BYTES.., "r") => "rFrb",
        (true, HUGE_SIZE_BYTES.., _) => "Frb",
        (true, LARGE_SIZE_BYTES.., "r") => "rFy",
        (true, LARGE_SIZE_BYTES.., _) => "Fy",
        (_, _, "r") => "r",
        _ => "",
    }
}

fn long_time_text(
    text: &str,
    mtime: SystemTime,
    params: &Params,
    color_level: LongFormatColorLevel,
) -> String {
    let age = match SystemTime::now().duration_since(mtime) {
        Ok(age) => age,
        Err(_) => return future_time_text(text, color_level),
    };

    if color_level == LongFormatColorLevel::None {
        return text.to_string();
    }

    if !params.time_gradient {
        return text.yellow().to_string();
    }

    if color_level == LongFormatColorLevel::Truecolor {
        return truecolor_time_text(text, age);
    }
    if color_level == LongFormatColorLevel::Ansi256 {
        return ansi_256_time_text(text, age);
    }

    named_time_text(text, age)
}

fn future_time_text(text: &str, color_level: LongFormatColorLevel) -> String {
    match color_level {
        LongFormatColorLevel::Truecolor => text.rgb(220, 80, 70).to_string(),
        LongFormatColorLevel::Ansi256 => {
            format!("\x1b[1;38;5;203m{text}\x1b[0m")
        }
        LongFormatColorLevel::Named => text.red().bold().to_string(),
        LongFormatColorLevel::None => text.to_string(),
    }
}

fn truecolor_time_text(text: &str, age: Duration) -> String {
    let (start, end, ratio) = time_color_segment(
        age,
        [
            (255, 209, 102),
            (236, 187, 82),
            (208, 159, 65),
            (150, 103, 38),
        ],
    );
    let red = interpolate(start.0, end.0, ratio);
    let green = interpolate(start.1, end.1, ratio);
    let blue = interpolate(start.2, end.2, ratio);

    text.rgb(red, green, blue).to_string()
}

fn ansi_256_time_text(text: &str, age: Duration) -> String {
    let (color, bold) = if age < DAY {
        (222, true)
    } else if age < WEEK {
        (221, false)
    } else if age < MONTH {
        (178, false)
    } else if age < YEAR {
        (136, false)
    } else {
        (130, false)
    };

    if bold {
        format!("\x1b[1;38;5;{color}m{text}\x1b[0m")
    } else {
        format!("\x1b[38;5;{color}m{text}\x1b[0m")
    }
}

fn named_time_text(text: &str, age: Duration) -> String {
    if age < DAY {
        text.yellow().bold().to_string()
    } else if age < YEAR {
        text.yellow().to_string()
    } else {
        text.yellow().dim().to_string()
    }
}

fn time_color_segment(
    age: Duration,
    colors: [(u8, u8, u8); 4],
) -> ((u8, u8, u8), (u8, u8, u8), f32) {
    if age < DAY {
        (colors[0], colors[0], 0.0)
    } else if age < WEEK {
        (colors[0], colors[1], segment_ratio(age, DAY, WEEK))
    } else if age < MONTH {
        (colors[1], colors[2], segment_ratio(age, WEEK, MONTH))
    } else if age < YEAR {
        (colors[2], colors[3], segment_ratio(age, MONTH, YEAR))
    } else {
        (colors[3], colors[3], 0.0)
    }
}

fn segment_ratio(age: Duration, start: Duration, end: Duration) -> f32 {
    let elapsed = age.saturating_sub(start).as_secs_f32();
    let span = end.saturating_sub(start).as_secs_f32();
    (elapsed / span).clamp(0.0, 1.0)
}

fn interpolate(start: u8, end: u8, ratio: f32) -> u8 {
    let ratio = ratio.clamp(0.0, 1.0);
    (f32::from(start) + (f32::from(end) - f32::from(start)) * ratio).round()
        as u8
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
    table.print_tty(false)?;
    io::stdout().flush()
}

fn print_short_lines(lines: &[String]) -> io::Result<()> {
    let mut stdout = io::stdout();
    for line in lines {
        writeln!(stdout, "{line}")?;
    }
    stdout.flush()
}
