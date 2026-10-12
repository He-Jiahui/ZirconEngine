//! 专用工作台 tooltip 在模板节点链中认领外壳和文字，复用共享状态样式与字体测量；显示时机由外部浮层状态控制。

#[path = "tooltips/commands.rs"]
mod commands;
#[path = "tooltips/identity.rs"]
mod identity;
#[path = "tooltips/layers.rs"]
mod layers;
#[path = "tooltips/layout.rs"]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod layout;
#[path = "tooltips/metrics.rs"]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod metrics;
#[path = "tooltips/surface.rs"]
mod surface;
#[path = "tooltips/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tooltip_commands;

#[cfg(test)]
#[path = "tooltips_tests/tests/mod.rs"]
mod tests;
