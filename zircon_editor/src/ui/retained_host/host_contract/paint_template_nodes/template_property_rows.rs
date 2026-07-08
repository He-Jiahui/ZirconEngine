mod commands;
mod fields;
mod identity;
mod labels;
mod layers;
mod layout;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_property_row_text_commands;

#[cfg(test)]
#[path = "template_property_rows_tests/mod.rs"]
mod tests;
