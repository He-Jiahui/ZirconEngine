//! 面板子命令的组织边界：统一暴露底面、搜索与空结果入口，供 commands 管理提交顺序。

mod empty;
mod entry;
mod search;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use empty::push_command_palette_empty_message;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_command_palette_panel_commands;
