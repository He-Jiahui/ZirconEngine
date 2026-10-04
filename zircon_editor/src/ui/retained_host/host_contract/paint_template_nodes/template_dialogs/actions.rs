//! 对话框动作绘制的组织边界；commands统一布局并返回正文需要避让的动作带位置。

mod commands;
mod labels;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dialog_actions;
