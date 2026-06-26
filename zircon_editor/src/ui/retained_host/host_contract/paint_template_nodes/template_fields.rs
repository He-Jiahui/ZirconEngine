mod commands;
mod geometry;
mod identity;
mod search;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_fields_tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_field_commands;
