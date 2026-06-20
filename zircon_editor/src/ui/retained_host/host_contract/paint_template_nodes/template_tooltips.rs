mod commands;
mod identity;
mod layout;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tooltip_commands;

#[cfg(test)]
use super::super::data::TemplatePaneNodeData;
#[cfg(test)]
use super::template_tooltip_glyphs::tooltip_arrow_size;

#[cfg(test)]
#[path = "template_tooltips_tests.rs"]
mod tests;
