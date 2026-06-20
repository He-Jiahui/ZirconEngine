mod actions;
mod commands;
mod identity;
mod labels;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tree_row_commands;
#[cfg(test)]
use identity::is_workbench_tree_row;
#[cfg(test)]
use style::tree_row_style;

#[cfg(test)]
#[path = "template_tree_rows_tests.rs"]
mod tests;
