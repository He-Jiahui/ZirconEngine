// 根框、图标、正文和操作区域共享节点框及裁剪边界，由此模块汇总给 Alert 命令序列。
mod action;
mod icon;
mod message;
mod metrics;
mod root;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use action::{
    alert_action_frame, alert_action_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon::{
    alert_icon_frame, alert_icon_mark_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use message::{
    alert_message_frame, alert_message_left, alert_message_right,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use root::alert_rect;
