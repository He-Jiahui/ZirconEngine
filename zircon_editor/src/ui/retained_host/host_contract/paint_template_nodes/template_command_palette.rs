//! 命令面板专用绘制边界：消费宿主节点快照，把面板与候选行提交给统一命令流。
//! 查询、筛选、导航和执行归上游交互链所有；绘制层只读投影结果。

mod commands;
mod identity;
mod layers;
mod layout;
mod palette;
mod panel;
mod rows;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_command_palette_commands;

#[cfg(test)]
#[path = "template_command_palette/tests/cases.rs"]
mod tests;
