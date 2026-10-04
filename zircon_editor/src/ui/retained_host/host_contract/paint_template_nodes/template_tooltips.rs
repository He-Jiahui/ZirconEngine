//! 专用工作台 tooltip 在模板节点链中认领外壳和文字，复用共享状态样式与字体测量；显示时机由外部浮层状态控制。

mod commands;
mod identity;
mod layers;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod layout;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod metrics;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tooltip_commands;

#[cfg(test)]
#[path = "template_tooltips_tests/tests/mod.rs"]
mod tests;
