mod commands;
mod identity;
mod inline;
mod layout;
mod toast;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_alert_commands;

#[cfg(test)]
use super::style_selector::{
    WorkbenchAlertTone as AlertTone, select_workbench_alert_style, select_workbench_toast_style,
};
#[cfg(test)]
use identity::{WorkbenchAlertKind, workbench_alert_kind};
#[cfg(test)]
use toast::toast_status_mark_size;

#[cfg(test)]
#[path = "template_alerts_tests/mod.rs"]
mod tests;
