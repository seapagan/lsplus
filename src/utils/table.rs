use prettytable::{Table, format::FormatBuilder};

pub fn create_table(padding: usize) -> Table {
    let format = FormatBuilder::new()
        .column_separator(' ')
        .left_border(' ')
        .padding(0, padding)
        .build();
    let mut table = Table::new();
    table.set_format(format);
    table
}
