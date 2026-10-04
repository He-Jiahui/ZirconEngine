//! 通知中心的专用绘制边界；行集合、未读数和省略数由投影链提供。
//! 本模块只绘制已打开面板及可见记录，不拥有通知队列、已读变更或操作执行。

mod commands;
mod identity;
#[cfg(test)]
#[path = "template_notification_center/tests/instrumentation.rs"]
mod instrumentation;
mod layout;
mod panel;
mod row;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_notification_center_commands;
