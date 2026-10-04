//! primary 专用链中的分段组和页签入口；组件家族决定是多选项组还是单个 tab。

mod commands;
mod identity;
mod labels;
mod options;
mod segments;
mod style;
mod tabs;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_segmented_control_commands;

#[cfg(test)]
#[path = "template_segmented_controls_tests/tests/mod.rs"]
mod tests;
