use prettytable::{format::FormatBuilder, Cell, Row, Table};

pub fn create_table(padding: usize) -> Table {
    let format = FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(0, padding)
        .build();
    let mut table = Table::new();
    table.set_format(format);
    table
}

pub fn add_files_to_table(
    table: &mut Table,
    file_names: &[String],
    num_columns: usize,
) {
    for chunk in file_names.chunks(num_columns) {
        let mut row = Row::empty();
        for cell in chunk.iter() {
            row.add_cell(Cell::new(cell));
        }
        table.add_row(row);
    }
}
