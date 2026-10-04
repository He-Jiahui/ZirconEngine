//! 候选行子命令的入口边界；一行的状态、匹配标记及两种文本由 entry 统一组合。

mod detail;
mod entry;
mod indicator;
mod label;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_command_row_commands;
