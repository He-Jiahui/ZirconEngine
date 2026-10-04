//! 通用 fallback 的属性行正文入口；它接管标签与值，使随后普通文字回退不会把整行重复叠画。

mod commands;
mod fields;
mod identity;
mod labels;
mod layers;
mod layout;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_property_row_text_commands;

#[cfg(test)]
#[path = "template_property_rows_tests/tests/mod.rs"]
mod tests;
