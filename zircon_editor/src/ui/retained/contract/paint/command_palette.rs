//! 命令面板专用绘制边界：消费宿主节点快照，把面板与候选行提交给统一命令流。
//! 查询、筛选、导航和执行归上游交互链所有；绘制层只读投影结果。

#[path = "command_palette/commands.rs"]
mod commands;
#[path = "command_palette/identity.rs"]
mod identity;
#[path = "command_palette/layers.rs"]
mod layers;
#[path = "command_palette/layout.rs"]
mod layout;
#[path = "command_palette/palette.rs"]
mod palette;
#[path = "command_palette/panel.rs"]
mod panel;
#[path = "command_palette/rows.rs"]
mod rows;
#[path = "command_palette/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_command_palette_commands;

#[cfg(test)]
#[path = "command_palette/tests/cases.rs"]
mod tests;
