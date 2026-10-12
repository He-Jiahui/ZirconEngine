//! 通用 fallback 的属性行正文入口；它接管标签与值，使随后普通文字回退不会把整行重复叠画。

#[path = "property_rows/commands.rs"]
mod commands;
#[path = "property_rows/fields.rs"]
mod fields;
#[path = "property_rows/identity.rs"]
mod identity;
#[path = "property_rows/labels.rs"]
mod labels;
#[path = "property_rows/layers.rs"]
mod layers;
#[path = "property_rows/layout.rs"]
mod layout;
#[path = "property_rows/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_property_row_text_commands;

#[cfg(test)]
#[path = "property_rows_tests/tests/mod.rs"]
mod tests;
