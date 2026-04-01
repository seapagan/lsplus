use chrono::{DateTime, Local};
use inline_colorization::*;
use prettytable::{Cell, Row};
use std::io;

use strip_ansi_escapes::strip_str;
use terminal_size::{Width, terminal_size};

use crate::Params;
use crate::structs::FileInfo;
use crate::utils;
use crate::utils::file::check_display_name;

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
    let max_name_length = file_info
        .iter()
        .map(|info| {
            let display_name = check_display_name(info);
            let clean_name = strip_str(&display_name);
            clean_name.len()
        })
        .max()
        .unwrap_or(0)
        + 2;

    let terminal_width = terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .unwrap_or(80);
    let num_columns = terminal_width / max_name_length;

    let mut table = utils::table::create_table(2);

    for chunk in file_info.chunks(num_columns) {
        let mut row = Row::empty();
        for info in chunk {
            let display_name = check_display_name(info);
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
