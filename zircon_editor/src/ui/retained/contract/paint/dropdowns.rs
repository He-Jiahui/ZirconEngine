//! 下拉框专用绘制入口和几何导出；template_nodes 的 dropdown 链随后以同一锚点安排弹出行。

#[path = "dropdowns/commands.rs"]
mod commands;
#[path = "dropdowns/geometry.rs"]
mod geometry;
#[path = "dropdowns/identity.rs"]
mod identity;
#[path = "dropdowns/layers.rs"]
mod layers;
#[path = "dropdowns/style.rs"]
mod style;
#[path = "dropdowns/surface.rs"]
mod surface;
#[path = "dropdowns/text.rs"]
mod text;

#[cfg(test)]
#[path = "dropdowns_tests/tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dropdown_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::dropdown_paint_rect;
