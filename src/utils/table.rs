use prettytable::{format::FormatBuilder, Table};

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
