//! 下拉框专用绘制入口和几何导出；template_nodes 的 dropdown 链随后以同一锚点安排弹出行。

mod commands;
mod geometry;
mod identity;
mod layers;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_dropdowns_tests/tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dropdown_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::dropdown_paint_rect;
