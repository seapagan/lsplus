//! Borderless table rendering with ANSI-aware cell width measurement.

use std::fmt;
use std::io::{self, Write};

use strip_ansi_escapes::strip_str;
use unicode_width::UnicodeWidthStr;

const SPACE_BUFFER: [u8; 64] = [b' '; 64];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Alignment {
    Left,
    Right,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Cell {
    text: String,
    width: usize,
    align: Alignment,
}

impl Cell {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self::with_alignment(text, Alignment::Left)
    }

    pub(crate) fn right(text: impl Into<String>) -> Self {
        Self::with_alignment(text, Alignment::Right)
    }

    fn with_alignment(text: impl Into<String>, align: Alignment) -> Self {
        let text = text.into();
        let width = visible_width(&text);
        Self { text, width, align }
    }

    fn write_padded(
        &self,
        output: &mut impl Write,
        target_width: usize,
        skip_right_fill: bool,
    ) -> io::Result<()> {
        let padding = target_width.saturating_sub(self.width);

        match self.align {
            Alignment::Left => {
                write!(output, "{}", self.text)?;
                if !skip_right_fill {
                    write_spaces(output, padding)?;
                }
            }
            Alignment::Right => {
                write_spaces(output, padding)?;
                write!(output, "{}", self.text)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Row {
    cells: Vec<Cell>,
}

impl Row {
    pub(crate) fn new(cells: Vec<Cell>) -> Self {
        Self { cells }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Table {
    header: Option<Row>,
    rows: Vec<Row>,
    default_gap: usize,
    column_gaps: Vec<usize>,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            header: None,
            rows: Vec::new(),
            default_gap: 1,
            column_gaps: Vec::new(),
        }
    }

    pub(crate) fn set_header(&mut self, header: Row) {
        self.header = Some(header);
    }

    pub(crate) fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    pub(crate) fn set_default_gap(&mut self, gap: usize) {
        self.default_gap = gap;
    }

    pub(crate) fn set_column_gap(&mut self, column: usize, gap: usize) {
        if self.column_gaps.len() <= column {
            self.column_gaps.resize(column + 1, self.default_gap);
        }
        self.column_gaps[column] = gap;
    }

    pub(crate) fn write_to(&self, output: &mut impl Write) -> io::Result<()> {
        let widths = self.column_widths();
        if let Some(header) = &self.header {
            self.write_row(output, header, &widths)?;
        }

        for row in &self.rows {
            self.write_row(output, row, &widths)?;
        }

        Ok(())
    }

    fn write_row(
        &self,
        output: &mut impl Write,
        row: &Row,
        widths: &[usize],
    ) -> io::Result<()> {
        write!(output, " ")?;
        for column in 0..widths.len() {
            let skip_right_fill = column == widths.len() - 1;
            if let Some(cell) = row.cells.get(column) {
                cell.write_padded(output, widths[column], skip_right_fill)?;
            } else if !skip_right_fill {
                write_spaces(output, widths[column])?;
            }

            if !skip_right_fill {
                write_spaces(output, self.gap_after(column))?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    fn gap_after(&self, column: usize) -> usize {
        self.column_gaps
            .get(column)
            .copied()
            .unwrap_or(self.default_gap)
    }

    fn column_widths(&self) -> Vec<usize> {
        let column_count = self
            .rows_for_widths()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);
        let mut widths = vec![0; column_count];

        for row in self.rows_for_widths() {
            for (index, cell) in row.cells.iter().enumerate() {
                widths[index] = widths[index].max(cell.width);
            }
        }

        widths
    }

    fn rows_for_widths(&self) -> impl Iterator<Item = &Row> {
        self.header.iter().chain(self.rows.iter())
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Table {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = Vec::new();
        self.write_to(&mut output).map_err(|_| fmt::Error)?;
        let rendered = String::from_utf8(output).map_err(|_| fmt::Error)?;
        formatter.write_str(&rendered)
    }
}

pub(crate) fn visible_width(text: &str) -> usize {
    let stripped = strip_str(text);
    UnicodeWidthStr::width(stripped.as_str())
}

fn write_spaces(output: &mut impl Write, count: usize) -> io::Result<()> {
    let mut remaining = count;
    while remaining > 0 {
        let chunk = remaining.min(SPACE_BUFFER.len());
        output.write_all(&SPACE_BUFFER[..chunk])?;
        remaining -= chunk;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Cell, Row, Table};

    #[test]
    fn table_header_contributes_to_widths_and_uses_column_gaps() {
        let mut table = Table::new();
        table.set_default_gap(2);
        table.set_column_gap(0, 1);
        table.set_header(Row::new(vec![
            Cell::new("Long Header"),
            Cell::new("Next"),
        ]));
        table.add_row(Row::new(vec![Cell::new("x"), Cell::new("y")]));

        assert_eq!(table.to_string(), " Long Header Next\n x           y\n");
    }

    #[test]
    fn table_does_not_pad_final_left_aligned_column() {
        let mut table = Table::new();
        table.add_row(Row::new(vec![Cell::new("a"), Cell::new("short")]));
        table.add_row(Row::new(vec![Cell::new("b"), Cell::new("longer")]));

        assert_eq!(table.to_string(), " a short\n b longer\n");
    }
}
