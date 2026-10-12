//! 对话框专用绘制边界；展示上游投影的标题、正文、动作与严重性，不拥有模态生命周期或操作执行。

#[path = "dialogs/actions.rs"]
mod actions;
#[path = "dialogs/commands.rs"]
mod commands;
#[path = "dialogs/content.rs"]
mod content;
#[path = "dialogs/identity.rs"]
mod identity;
#[path = "dialogs/layout.rs"]
mod layout;
#[path = "dialogs/metrics.rs"]
mod metrics;
#[path = "dialogs/style.rs"]
mod style;
#[path = "dialogs/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dialog_commands;

#[cfg(test)]
#[path = "dialogs/tests/cases.rs"]
mod tests;
