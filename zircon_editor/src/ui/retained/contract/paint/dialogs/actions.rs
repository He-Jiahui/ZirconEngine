//! 对话框动作绘制的组织边界；commands统一布局并返回正文需要避让的动作带位置。

#[path = "actions/commands.rs"]
mod commands;
#[path = "actions/labels.rs"]
mod labels;
#[path = "actions/surface.rs"]
mod surface;
#[path = "actions/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dialog_actions;
