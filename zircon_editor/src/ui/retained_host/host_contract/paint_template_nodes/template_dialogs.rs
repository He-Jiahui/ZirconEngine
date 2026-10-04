//! 对话框专用绘制边界；展示上游投影的标题、正文、动作与严重性，不拥有模态生命周期或操作执行。

mod actions;
mod commands;
mod content;
mod identity;
mod layout;
mod metrics;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dialog_commands;

#[cfg(test)]
#[path = "template_dialogs/tests/cases.rs"]
mod tests;
