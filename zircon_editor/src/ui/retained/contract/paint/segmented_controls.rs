//! primary 专用链中的分段组和页签入口；组件家族决定是多选项组还是单个 tab。

#[path = "segmented_controls/commands.rs"]
mod commands;
#[path = "segmented_controls/identity.rs"]
mod identity;
#[path = "segmented_controls/labels.rs"]
mod labels;
#[path = "segmented_controls/options.rs"]
mod options;
#[path = "segmented_controls/segments.rs"]
mod segments;
#[path = "segmented_controls/style.rs"]
mod style;
#[path = "segmented_controls/tabs.rs"]
mod tabs;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_segmented_control_commands;

#[cfg(test)]
#[path = "../../../contract/paint/tests/segmented_controls_tests_mod.rs"]
mod tests;
