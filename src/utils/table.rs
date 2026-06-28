//! Borderless table rendering with ANSI-aware cell width measurement.

use std::fmt;
use std::io::{self, Write};

use strip_ansi_escapes::strip_str;
use unicode_width::UnicodeWidthStr;

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Table {
    rows: Vec<Row>,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    pub(crate) fn write_to(&self, output: &mut impl Write) -> io::Result<()> {
        let widths = self.column_widths();
        for row in &self.rows {
            write!(output, " ")?;
            for column in 0..widths.len() {
                if column > 0 {
                    write!(output, " ")?;
                }

                let skip_right_fill = column == widths.len() - 1;
                if let Some(cell) = row.cells.get(column) {
                    cell.write_padded(
                        output,
                        widths[column],
                        skip_right_fill,
                    )?;
                } else if !skip_right_fill {
                    write_spaces(output, widths[column])?;
                }
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn column_widths(&self) -> Vec<usize> {
        let column_count = self
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);
        let mut widths = vec![0; column_count];

        for row in &self.rows {
            for (index, cell) in row.cells.iter().enumerate() {
                widths[index] = widths[index].max(cell.width);
            }
        }

        widths
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

fn visible_width(text: &str) -> usize {
    let stripped = strip_str(text);
    UnicodeWidthStr::width(stripped.as_str())
}

fn write_spaces(output: &mut impl Write, count: usize) -> io::Result<()> {
    for _ in 0..count {
        write!(output, " ")?;
    }
    Ok(())
}
