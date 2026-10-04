//! toast 子命令组织边界；entry 为本组件统一分发底面、消息和尾部反馈。

mod action;
mod entry;
mod icon;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_toast;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon::toast_status_mark_size;
