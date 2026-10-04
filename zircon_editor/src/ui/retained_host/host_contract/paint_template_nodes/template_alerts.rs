//! Workbench 提示条与 toast 的专用绘制边界；内容、级别和状态归宿主节点投影所有。
//! 本模块只把语义状态呈现为统一命令流，不执行撤销、关闭或通知生命周期操作。

mod commands;
mod identity;
mod inline;
mod layout;
mod toast;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_alert_commands;

#[cfg(test)]
use super::style_selector::{
    select_workbench_alert_style, select_workbench_toast_style, WorkbenchAlertTone as AlertTone,
};
#[cfg(test)]
use identity::{workbench_alert_kind, WorkbenchAlertKind};
#[cfg(test)]
use toast::toast_status_mark_size;

#[cfg(test)]
#[path = "template_alerts_tests/tests/mod.rs"]
mod tests;
