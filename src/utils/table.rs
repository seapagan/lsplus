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
pub(crate) struct HeaderCell {
    cell: Cell,
    span: usize,
}

impl HeaderCell {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self::from_cell(Cell::new(text))
    }

    pub(crate) fn right(text: impl Into<String>) -> Self {
        Self::from_cell(Cell::right(text))
    }

    pub(crate) fn span(mut self, span: usize) -> Self {
        self.span = span.max(1);
        self
    }

    fn from_cell(cell: Cell) -> Self {
        Self { cell, span: 1 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderRow {
    cells: Vec<HeaderCell>,
}

impl HeaderRow {
    pub(crate) fn new(cells: Vec<HeaderCell>) -> Self {
        Self { cells }
    }

    fn column_count(&self) -> usize {
        self.cells.iter().map(|cell| cell.span).sum()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Table {
    header: Option<HeaderRow>,
    rows: Vec<Row>,
    default_gap: usize,
    column_gaps: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ColumnSpan {
    start: usize,
    len: usize,
}

impl ColumnSpan {
    fn end(self) -> usize {
        self.start + self.len
    }

    fn last_column(self) -> usize {
        self.end() - 1
    }

    fn contains_final_column(self, widths: &[usize]) -> bool {
        self.end() >= widths.len()
    }
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

    pub(crate) fn set_header(&mut self, header: HeaderRow) {
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
            self.write_header(output, header, &widths)?;
        }

        for row in &self.rows {
            self.write_row(output, row, &widths)?;
        }

        Ok(())
    }

    fn write_header(
        &self,
        output: &mut impl Write,
        header: &HeaderRow,
        widths: &[usize],
    ) -> io::Result<()> {
        write!(output, " ")?;

        let mut column = 0;
        for header_cell in &header.cells {
            debug_assert!(
                column < widths.len(),
                "header column span exceeded computed widths"
            );
            let span = self.column_span(column, header_cell.span, widths);
            let target_width = self.spanned_width(widths, span);
            let skip_right_fill =
                span.len == 1 && span.contains_final_column(widths);
            header_cell.cell.write_padded(
                output,
                target_width,
                skip_right_fill,
            )?;

            column = span.end();
            if column < widths.len() {
                write_spaces(output, self.gap_after(column - 1))?;
            }
        }

        while column < widths.len() {
            let skip_right_fill = column == widths.len() - 1;
            if !skip_right_fill {
                write_spaces(output, widths[column])?;
                write_spaces(output, self.gap_after(column))?;
            }
            column += 1;
        }

        writeln!(output)?;
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

    fn column_span(
        &self,
        start: usize,
        requested_len: usize,
        widths: &[usize],
    ) -> ColumnSpan {
        ColumnSpan {
            start,
            len: requested_len.min(widths.len() - start),
        }
    }

    fn spanned_width(&self, widths: &[usize], span: ColumnSpan) -> usize {
        let content_width =
            widths[span.start..span.end()].iter().sum::<usize>();
        let gap_width = (span.start..span.last_column())
            .map(|column| self.gap_after(column))
            .sum::<usize>();
        content_width + gap_width
    }

    fn column_widths(&self) -> Vec<usize> {
        let row_column_count = self
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);
        let header_column_count = self
            .header
            .as_ref()
            .map_or(0, |header| header.column_count());
        let column_count = row_column_count.max(header_column_count);
        let mut widths = vec![0; column_count];

        for row in &self.rows {
            for (index, cell) in row.cells.iter().enumerate() {
                widths[index] = widths[index].max(cell.width);
            }
        }

        if let Some(header) = &self.header {
            let mut column = 0;
            for header_cell in &header.cells {
                debug_assert!(
                    column < widths.len(),
                    "header column span exceeded computed widths"
                );
                let span = self.column_span(column, header_cell.span, &widths);
                let target_width = self.spanned_width(&widths, span);
                if header_cell.cell.width > target_width {
                    let overflow = header_cell.cell.width - target_width;
                    let overflow_column = match header_cell.cell.align {
                        Alignment::Left => span.start,
                        Alignment::Right => span.last_column(),
                    };
                    widths[overflow_column] += overflow;
                }
                column = span.end();
            }
        }

        widths
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
    use super::{Cell, HeaderCell, HeaderRow, Row, Table};
    use std::io::{self, Cursor};

    #[test]
    fn table_header_contributes_to_widths_and_uses_column_gaps() {
        let mut table = Table::new();
        table.set_default_gap(2);
        table.set_column_gap(0, 1);
        table.set_header(HeaderRow::new(vec![
            HeaderCell::new("Long Header"),
            HeaderCell::new("Next"),
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

    #[test]
    fn table_header_cell_can_span_existing_columns() {
        let mut table = Table::new();
        table.set_default_gap(2);
        table.set_column_gap(0, 1);
        table.set_header(HeaderRow::new(vec![
            HeaderCell::right("Size").span(2),
            HeaderCell::new("Name"),
        ]));
        table.add_row(Row::new(vec![
            Cell::right("4"),
            Cell::new("K"),
            Cell::new("file"),
        ]));

        assert_eq!(table.to_string(), " Size  Name\n 4 K   file\n");
    }

    #[test]
    fn table_left_aligned_spanned_header_expands_first_column() {
        let mut table = Table::new();
        table.set_header(HeaderRow::new(vec![
            HeaderCell::new("Label").span(2),
            HeaderCell::new("Name"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("x"),
            Cell::new("y"),
            Cell::new("file"),
        ]));

        assert_eq!(table.to_string(), " Label Name\n x   y file\n");
    }

    #[test]
    fn table_header_cell_spanning_final_columns_keeps_interior_padding() {
        let mut table = Table::new();
        table.set_header(HeaderRow::new(vec![HeaderCell::new("Hi").span(2)]));
        table.add_row(Row::new(vec![Cell::new("x"), Cell::new("y")]));

        assert_eq!(table.to_string(), " Hi \n x y\n");
    }

    #[test]
    fn table_header_only_output_keeps_header_content() {
        let mut table = Table::new();
        table.set_header(HeaderRow::new(vec![
            HeaderCell::new("Name"),
            HeaderCell::right("Size"),
        ]));

        assert_eq!(table.to_string(), " Name Size\n");
    }

    #[test]
    fn table_default_matches_new_table() {
        assert_eq!(Table::default(), Table::new());
    }

    #[test]
    fn table_pads_missing_header_columns_before_final_column() {
        let mut table = Table::new();
        table.set_header(HeaderRow::new(vec![HeaderCell::new("Name")]));
        table.add_row(Row::new(vec![
            Cell::new("a"),
            Cell::new("middle"),
            Cell::new("end"),
        ]));

        assert_eq!(table.to_string(), " Name        \n a    middle end\n");
    }

    #[test]
    fn table_pads_missing_row_cells_before_final_column() {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("a"),
            Cell::new("wide"),
            Cell::new("end"),
        ]));
        table.add_row(Row::new(vec![Cell::new("b")]));

        assert_eq!(table.to_string(), " a wide end\n b      \n");
    }

    #[test]
    fn table_propagates_header_cell_write_errors() {
        let table = Table::new();
        let header = HeaderRow::new(vec![HeaderCell::new("Name")]);
        // One byte lets the leading-space write succeed, then the header cell fails.
        let mut output = Cursor::new([0_u8; 1]);

        let err = table.write_header(&mut output, &header, &[4]).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::WriteZero);
    }
}
