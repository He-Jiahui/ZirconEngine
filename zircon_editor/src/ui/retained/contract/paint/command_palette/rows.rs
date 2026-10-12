//! 候选行子命令的入口边界；一行的状态、匹配标记及两种文本由 entry 统一组合。

#[path = "rows/detail.rs"]
mod detail;
#[path = "rows/entry.rs"]
mod entry;
#[path = "rows/indicator.rs"]
mod indicator;
#[path = "rows/label.rs"]
mod label;
#[path = "rows/style.rs"]
mod style;
#[path = "rows/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_command_row_commands;
