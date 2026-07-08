mod commands;
mod geometry;
mod identity;
mod layers;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_dropdowns_tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dropdown_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::dropdown_paint_rect;
