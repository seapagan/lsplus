//! Output rendering for long and short listing formats.
//!
//! Long and short formats compute terminal-width-aware columns using visible
//! Unicode width so ANSI styling and wide glyphs do not distort layout.

use chrono::{DateTime, Local};
use colored_text::{ColorLevel, Colorize, StyledText};
use std::fmt::Write as FmtWrite;
use std::io::{self, IsTerminal, Write as IoWrite};
use std::time::{Duration, SystemTime};

use strip_ansi_escapes::strip_str;
use term_grid::{Direction, Filling, Grid, GridOptions};
use terminal_size::{Height, Width, terminal_size};

use crate::Params;
use crate::platform::{self, LongColumn, LongFormatLayoutOptions};
use crate::structs::{AttributeDisplay, FileInfo, NameStyle, ShortFormat};
use crate::utils;
use crate::utils::color::long_format_color_level;
use crate::utils::file::check_display_name;
use crate::utils::table::{Cell, HeaderCell, HeaderRow, Row, Table};
use crate::utils::time::{DAY, MONTH, WEEK, YEAR};

const SHORT_COLUMN_GAP: usize = 2;
const LONG_TABLE_DEFAULT_GAP: usize = 2;
const LONG_TABLE_RELATED_GAP: usize = 1;
const LARGE_SIZE_BYTES: u64 = 1024 * 1024;
const HUGE_SIZE_BYTES: u64 = 1024 * 1024 * 1024;
const HEADER_SALMON_TRUECOLOR: (u8, u8, u8) = (250, 128, 114);
const HEADER_SALMON_ANSI_256: u8 = 209;

fn long_column_header(
    column: LongColumn,
    attributes: AttributeDisplay,
) -> &'static str {
    match column {
        LongColumn::UnixSymbolicPermissions
        | LongColumn::UnixOctalWithType => "Permissions",
        LongColumn::UnixOctal => "Octal",
        LongColumn::Type => "Type",
        LongColumn::Attributes => match attributes {
            AttributeDisplay::Long | AttributeDisplay::Short => "Attributes",
            AttributeDisplay::Minimal => "Attr",
        },
        LongColumn::Links => "Links",
        LongColumn::User => "User",
        LongColumn::Group => "Group",
        LongColumn::Size => "Size",
        LongColumn::Unit | LongColumn::Icon => "",
        LongColumn::Date => "Date Modified",
        LongColumn::Name => "Name",
    }
}

fn long_column_aligns_right(column: LongColumn) -> bool {
    matches!(column, LongColumn::Size | LongColumn::Date)
}

/// Render long-format rows to stdout.
pub fn display_long_format(
    file_info: &[FileInfo],
    params: &Params,
) -> io::Result<()> {
    print_table(&build_long_format_table(file_info, params))
}

/// Style a directory section header.
pub(crate) fn directory_header_text(header: &str) -> String {
    header.blue().bold().to_string()
}

/// Render long-format rows with prefixes prepended to the name column.
pub(crate) fn display_long_format_with_name_prefixes<'a>(
    file_info: impl IntoIterator<Item = (&'a FileInfo, &'a str)>,
    params: &Params,
) -> io::Result<()> {
    print_table(&build_long_format_table_with_name_prefixes(
        file_info, params,
    ))
}

/// Build the long-format table without printing it.
pub(crate) fn build_long_format_table(
    file_info: &[FileInfo],
    params: &Params,
) -> Table {
    build_long_format_table_with_name_prefixes(
        file_info.iter().map(|info| (info, "")),
        params,
    )
}

pub(crate) fn build_long_format_table_with_name_prefixes<'a>(
    file_info: impl IntoIterator<Item = (&'a FileInfo, &'a str)>,
    params: &Params,
) -> Table {
    let mut table = Table::new();
    table.set_default_gap(LONG_TABLE_DEFAULT_GAP);
    let color_level = long_format_color_level(params);
    let entries: Vec<_> = file_info.into_iter().collect();
    let columns = long_format_columns(params);
    apply_long_format_gaps(&mut table, &columns);

    if params.header && !entries.is_empty() {
        table.set_header(long_format_header_row(
            &columns,
            params.attributes,
            color_level,
        ));
    }

    for (info, name_prefix) in entries {
        table.add_row(long_format_row(
            info,
            name_prefix,
            params,
            color_level,
            &columns,
        ));
    }

    table
}

fn long_format_columns(params: &Params) -> Vec<LongColumn> {
    platform::long_format_layout(&LongFormatLayoutOptions {
        permission_display: params.permissions,
        include_size_unit: params.size_scale().is_some(),
        include_icon: !params.no_icons,
    })
    .columns
}

fn apply_long_format_gaps(table: &mut Table, columns: &[LongColumn]) {
    for (index, pair) in columns.windows(2).enumerate() {
        if matches!(
            pair,
            [LongColumn::UnixSymbolicPermissions, LongColumn::UnixOctal]
                | [LongColumn::User, LongColumn::Group]
                | [LongColumn::Size, LongColumn::Unit]
        ) {
            table.set_column_gap(index, LONG_TABLE_RELATED_GAP);
        }
    }
}

fn long_format_row(
    info: &FileInfo,
    name_prefix: &str,
    params: &Params,
    color_level: ColorLevel,
    columns: &[LongColumn],
) -> Row {
    let display_time = if params.fuzzy_time {
        utils::fuzzy_time(info.mtime).to_string()
    } else {
        let datetime: DateTime<Local> = DateTime::from(info.mtime);
        datetime.format("%c").to_string()
    };
    let size_scale = params.size_scale();
    let (display_size, units) =
        utils::format::show_size(info.size, size_scale);
    let display_name = format!("{}{}", name_prefix, check_display_name(info));
    let mut cells = Vec::with_capacity(columns.len());

    for column in columns {
        cells.push(match column {
            LongColumn::UnixSymbolicPermissions => {
                symbolic_permission_cell(info, params, color_level)
            }
            LongColumn::UnixOctalWithType => {
                octal_with_type_permission_cell(info, params, color_level)
            }
            LongColumn::UnixOctal => {
                octal_permission_cell(info, params, color_level)
            }
            LongColumn::Type => {
                Cell::new(long_file_type_text(info, params, color_level))
            }
            LongColumn::Attributes => Cell::new(info.mode.clone()),
            LongColumn::Links => Cell::new(info.nlink.to_string()),
            LongColumn::User => Cell::new(info.user.cyan().to_string()),
            LongColumn::Group => Cell::new(info.group.green().to_string()),
            LongColumn::Size => {
                size_cell(&display_size, info.size, params, color_level, true)
            }
            LongColumn::Unit => {
                size_cell(units, info.size, params, color_level, false)
            }
            LongColumn::Date => Cell::right(long_time_text(
                &display_time,
                info.mtime,
                params,
                color_level,
            )),
            LongColumn::Icon => icon_cell(info),
            LongColumn::Name => Cell::new(&display_name),
        });
    }

    Row::new(cells)
}

fn long_format_header_row(
    columns: &[LongColumn],
    attributes: AttributeDisplay,
    color_level: ColorLevel,
) -> HeaderRow {
    let mut cells = Vec::with_capacity(columns.len());
    let mut index = 0;

    while index < columns.len() {
        let column = columns[index];
        if matches!(column, LongColumn::Size)
            && columns.get(index + 1) == Some(&LongColumn::Unit)
        {
            cells.push(header_cell(column, attributes, color_level).span(2));
            index += 2;
        } else {
            cells.push(header_cell(column, attributes, color_level));
            index += 1;
        }
    }

    HeaderRow::new(cells)
}

fn header_cell(
    column: LongColumn,
    attributes: AttributeDisplay,
    color_level: ColorLevel,
) -> HeaderCell {
    let text =
        header_text(long_column_header(column, attributes), color_level);
    if long_column_aligns_right(column) {
        HeaderCell::right(text)
    } else {
        HeaderCell::new(text)
    }
}

fn header_text(text: &str, color_level: ColorLevel) -> String {
    if text.is_empty() {
        return String::new();
    }

    match color_level {
        ColorLevel::TrueColor => text
            .rgb(
                HEADER_SALMON_TRUECOLOR.0,
                HEADER_SALMON_TRUECOLOR.1,
                HEADER_SALMON_TRUECOLOR.2,
            )
            .underline()
            .to_string(),
        ColorLevel::Ansi256 => {
            text.ansi256(HEADER_SALMON_ANSI_256).underline().to_string()
        }
        ColorLevel::Ansi16 => text.red().underline().to_string(),
        ColorLevel::NoColor => text.to_string(),
    }
}

fn symbolic_permission_cell(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> Cell {
    Cell::new(long_permission_text(info, params, color_level))
}

fn octal_with_type_permission_cell(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> Cell {
    Cell::new(format!(
        "{} {}",
        long_file_type_text(info, params, color_level),
        long_octal_permission_text(info, params, color_level)
    ))
}

fn octal_permission_cell(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> Cell {
    Cell::new(long_octal_permission_text(info, params, color_level))
}

fn icon_cell(info: &FileInfo) -> Cell {
    info.item_icon
        .as_ref()
        .map_or_else(|| Cell::new(""), |icon| Cell::new(icon.to_string()))
}

fn long_file_type_text(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> String {
    if !params.permission_colors || color_level == ColorLevel::NoColor {
        return info.file_type.clone();
    }

    let mut output = String::with_capacity(info.file_type.len());
    for value in info.file_type.chars() {
        write_file_type_char(&mut output, value);
    }
    output
}

fn long_octal_permission_text(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> String {
    let text = utils::format::mode_to_octal(info.mode_bits);
    if !params.permission_colors || color_level == ColorLevel::NoColor {
        return text;
    }

    if color_level == ColorLevel::TrueColor {
        text.rgb(238, 204, 92).to_string()
    } else if color_level == ColorLevel::Ansi256 {
        text.ansi256(221).to_string()
    } else {
        text.yellow().dim().to_string()
    }
}

fn long_permission_text(
    info: &FileInfo,
    params: &Params,
    color_level: ColorLevel,
) -> String {
    if !params.permission_colors || color_level == ColorLevel::NoColor {
        return format!("{}{}", info.file_type, info.mode);
    }

    let mut output = long_file_type_text(info, params, color_level);
    output.reserve(info.mode.len());
    for value in info.mode.chars() {
        write_permission_char(&mut output, value);
    }

    output
}

fn write_file_type_char(output: &mut String, value: char) {
    match value {
        'd' => write!(output, "{}", value.blue()).unwrap(),
        'l' | 'L' => write!(output, "{}", value.cyan()).unwrap(),
        'j' => write!(output, "{}", value.magenta()).unwrap(),
        's' => write!(output, "{}", value.magenta().bold()).unwrap(),
        'p' => write!(output, "{}", value.yellow()).unwrap(),
        'c' | 'b' => write!(output, "{}", value.yellow().bold()).unwrap(),
        '-' | '?' | 'r' => write!(output, "{}", value.dim()).unwrap(),
        _ => output.push(value),
    }
}

fn write_permission_char(output: &mut String, value: char) {
    match value {
        'r' => write!(output, "{}", value.green()).unwrap(),
        'w' => write!(output, "{}", value.yellow()).unwrap(),
        'x' | 's' | 't' => write!(output, "{}", value.red().bold()).unwrap(),
        '-' | 'S' | 'T' => write!(output, "{}", value.dim()).unwrap(),
        _ => output.push(value),
    }
}

fn size_cell(
    text: &str,
    size: u64,
    params: &Params,
    color_level: ColorLevel,
    align_right: bool,
) -> Cell {
    let style =
        size_style_for_color_level(size, params, color_level, align_right);
    let text = style.format(text);
    if style.align_right() {
        Cell::right(text)
    } else {
        Cell::new(text)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SizeCellStyle {
    Plain,
    PlainRight,
    Large,
    LargeRight,
    Huge,
    HugeRight,
}

impl SizeCellStyle {
    fn align_right(self) -> bool {
        matches!(
            self,
            SizeCellStyle::PlainRight
                | SizeCellStyle::LargeRight
                | SizeCellStyle::HugeRight
        )
    }

    fn format(self, text: &str) -> String {
        match self {
            SizeCellStyle::Huge | SizeCellStyle::HugeRight => {
                text.red().bold().to_string()
            }
            SizeCellStyle::Large | SizeCellStyle::LargeRight => {
                text.yellow().to_string()
            }
            SizeCellStyle::Plain | SizeCellStyle::PlainRight => {
                text.to_string()
            }
        }
    }
}

/// Return the color and alignment style for a size cell.
pub(crate) fn size_style_for_color_level(
    size: u64,
    params: &Params,
    color_level: ColorLevel,
    align_right: bool,
) -> SizeCellStyle {
    match (
        params.size_colors && color_level != ColorLevel::NoColor,
        size,
        align_right,
    ) {
        (true, HUGE_SIZE_BYTES.., true) => SizeCellStyle::HugeRight,
        (true, HUGE_SIZE_BYTES.., _) => SizeCellStyle::Huge,
        (true, LARGE_SIZE_BYTES.., true) => SizeCellStyle::LargeRight,
        (true, LARGE_SIZE_BYTES.., _) => SizeCellStyle::Large,
        (_, _, true) => SizeCellStyle::PlainRight,
        _ => SizeCellStyle::Plain,
    }
}

/// Apply timestamp coloring according to age and terminal capability.
fn long_time_text(
    text: &str,
    mtime: SystemTime,
    params: &Params,
    color_level: ColorLevel,
) -> String {
    let age = match SystemTime::now().duration_since(mtime) {
        Ok(age) => age,
        Err(_) => return future_time_text(text, color_level),
    };

    if color_level == ColorLevel::NoColor {
        return text.to_string();
    }

    if !params.time_gradient {
        return text.yellow().to_string();
    }

    if color_level == ColorLevel::TrueColor {
        return truecolor_time_text(text, age);
    }
    if color_level == ColorLevel::Ansi256 {
        return ansi_256_time_text(text, age);
    }

    named_time_text(text, age)
}

fn future_time_text(text: &str, color_level: ColorLevel) -> String {
    match color_level {
        ColorLevel::TrueColor => text.rgb(220, 80, 70).to_string(),
        ColorLevel::Ansi256 => text.ansi256(203).bold().to_string(),
        ColorLevel::Ansi16 => text.red().bold().to_string(),
        ColorLevel::NoColor => text.to_string(),
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
        text.ansi256(color).bold().to_string()
    } else {
        text.ansi256(color).to_string()
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

/// Render short-format output to stdout.
pub fn display_short_format(
    file_info: &[FileInfo],
    params: &Params,
) -> io::Result<()> {
    let lines = if short_output_uses_grid(
        io::stdout().is_terminal(),
        params.short_format,
    ) {
        let terminal_width = terminal_width_or_default(terminal_size());
        render_short_format_lines(file_info, terminal_width)
    } else {
        render_short_single_column_lines(file_info)
    };

    print_short_lines(&lines)
}

/// Render short-format rows for a fixed terminal width.
pub(crate) fn render_short_format_lines(
    file_info: &[FileInfo],
    terminal_width: usize,
) -> Vec<String> {
    Grid::new(
        short_render_cells(file_info),
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(SHORT_COLUMN_GAP),
            width: terminal_width,
        },
    )
    .to_string()
    .lines()
    .map(str::to_owned)
    .collect()
}

/// Render one unpadded short-format entry per line.
pub(crate) fn render_short_single_column_lines(
    file_info: &[FileInfo],
) -> Vec<String> {
    short_render_cells(file_info)
}

/// Return whether short output should use a grid for this stdout context.
pub(crate) fn short_output_uses_grid(
    is_terminal: bool,
    short_format: Option<ShortFormat>,
) -> bool {
    is_terminal || short_format == Some(ShortFormat::Vertical)
}

fn short_render_cells(file_info: &[FileInfo]) -> Vec<String> {
    file_info.iter().map(short_render_cell).collect()
}

/// Return the detected terminal width, or the standard 80-column fallback.
pub(crate) fn terminal_width_or_default(
    size: Option<(Width, Height)>,
) -> usize {
    size.map(|(Width(width), _)| usize::from(width))
        .unwrap_or(80)
}

fn short_render_cell(info: &FileInfo) -> String {
    let display_name = check_display_name(info);
    let (prefix, name) = short_cell_parts(info, &display_name);
    format!("{prefix}{}", style_short_segment(info, name))
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

fn style_short_segment(info: &FileInfo, text: String) -> String {
    let styled = match info.name_style {
        NameStyle::Plain => StyledText::plain(text),
        NameStyle::Directory => text.blue(),
        NameStyle::Symlink => text.cyan(),
        NameStyle::Junction => text.magenta(),
        NameStyle::Executable => text.green().bold(),
        NameStyle::Socket => text.magenta().bold(),
        NameStyle::Fifo => text.yellow(),
        NameStyle::CharDevice | NameStyle::BlockDevice => text.yellow().bold(),
    };

    if info.dimmed {
        styled.dim().to_string()
    } else {
        styled.to_string()
    }
}

fn print_table(table: &Table) -> io::Result<()> {
    // Long-format colors are decided before cells reach the table renderer.
    // Keep raw ANSI escapes gated by long_format_color_level().
    let mut stdout = io::stdout();
    table.write_to(&mut stdout)?;
    stdout.flush()
}

fn print_short_lines(lines: &[String]) -> io::Result<()> {
    let mut stdout = io::stdout();
    for line in lines {
        writeln!(stdout, "{line}")?;
    }
    stdout.flush()
}
