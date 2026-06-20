mod actions;
mod cells;
mod commands;
mod identity;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::{
    push_table_row_commands, push_table_row_text_commands,
};

#[cfg(test)]
#[path = "template_table_rows_tests.rs"]
mod tests;
