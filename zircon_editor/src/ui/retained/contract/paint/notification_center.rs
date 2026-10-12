//! 通知中心的专用绘制边界；行集合、未读数和省略数由投影链提供。
//! 本模块只绘制已打开面板及可见记录，不拥有通知队列、已读变更或操作执行。

#[path = "notification_center/commands.rs"]
mod commands;
#[path = "notification_center/identity.rs"]
mod identity;
#[cfg(test)]
#[path = "tests/instrumentation.rs"]
mod instrumentation;
#[path = "notification_center/layout.rs"]
mod layout;
#[path = "notification_center/panel.rs"]
mod panel;
#[path = "notification_center/row.rs"]
mod row;
#[path = "notification_center/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_notification_center_commands;
