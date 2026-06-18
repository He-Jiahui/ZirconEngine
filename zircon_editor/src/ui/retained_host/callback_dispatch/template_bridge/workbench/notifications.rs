use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::retained_host::workbench_notifications::{
    WorkbenchNotification, WorkbenchNotificationSeverity,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

pub(crate) const WORKBENCH_TOAST_CONTROL_ID: &str = "WorkbenchToast";
pub(crate) const WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID: &str = "WorkbenchNotificationCenter";

const MAX_NOTIFICATION_HISTORY: usize = 64;
const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const FOCUSED: &str = "focused";
const SELECTED: &str = "selected";
const TEXT: &str = "text";
const VALUE: &str = "value";
const VALUE_TEXT: &str = "value_text";
const SEVERITY: &str = "severity";
const COLOR: &str = "color";
const ICON: &str = "icon";
const ACTION_LABEL: &str = "action_label";
const ACTION_COLOR: &str = "action_color";
const TOAST_QUEUE: &str = "toast_queue";
const CURRENT_TOAST_ID: &str = "current_toast_id";
const EXPIRED_TOAST_ID: &str = "expired_toast_id";
const QUEUE_LENGTH: &str = "queue_length";
const AUTO_HIDE_DURATION_MS: &str = "auto_hide_duration_ms";
const NOTIFICATIONS: &str = "notifications";
const UNREAD_COUNT: &str = "unread_count";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const FOCUSED_INDEX: &str = "focused_index";
const INPUT_INTERACTIVE: &str = "input_interactive";
const INPUT_CLICKABLE: &str = "input_clickable";
const INPUT_HOVERABLE: &str = "input_hoverable";
const INPUT_FOCUSABLE: &str = "input_focusable";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn push_workbench_notifications(
        &mut self,
        notifications: &[WorkbenchNotification],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if notifications.is_empty() {
            return Ok(false);
        }

        let mut changed = false;
        if self.has_control(WORKBENCH_TOAST_CONTROL_ID) {
            self.push_toast_queue(notifications)?;
            changed = true;
        }
        if self.has_control(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID) {
            self.push_notification_history(notifications)?;
            changed = true;
        }

        if changed {
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
        }
        Ok(changed)
    }

    fn push_toast_queue(
        &mut self,
        notifications: &[WorkbenchNotification],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let queue = notifications
            .iter()
            .map(WorkbenchNotification::toast_queue_entry)
            .collect::<Vec<_>>();
        let active = &notifications[0];

        self.set_visible(WORKBENCH_TOAST_CONTROL_ID, true)?;
        for property in [OPEN, POPUP_OPEN] {
            self.mutate_control_property(
                WORKBENCH_TOAST_CONTROL_ID,
                property,
                UiValue::Bool(true),
            )?;
        }
        self.mutate_control_property(WORKBENCH_TOAST_CONTROL_ID, FOCUSED, UiValue::Bool(false))?;
        self.mutate_control_property(WORKBENCH_TOAST_CONTROL_ID, SELECTED, UiValue::Bool(false))?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            TEXT,
            UiValue::String(active.toast_message().to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            VALUE,
            UiValue::String(active.id.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            VALUE_TEXT,
            UiValue::String(active.toast_message().to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            SEVERITY,
            UiValue::String(active.severity.as_str().to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            COLOR,
            UiValue::String(active.severity.as_str().to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ICON,
            UiValue::String(active.severity.icon().to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ACTION_LABEL,
            UiValue::String(active.action_label.clone().unwrap_or_default()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ACTION_COLOR,
            UiValue::String(severity_action_color(active.severity).to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            TOAST_QUEUE,
            string_array_value(queue),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            CURRENT_TOAST_ID,
            UiValue::String(active.id.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            EXPIRED_TOAST_ID,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            QUEUE_LENGTH,
            UiValue::Int(notifications.len() as i64),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            AUTO_HIDE_DURATION_MS,
            UiValue::Int(active.auto_hide_duration_ms),
        )?;
        Ok(())
    }

    fn push_notification_history(
        &mut self,
        notifications: &[WorkbenchNotification],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let mut history = notifications
            .iter()
            .map(WorkbenchNotification::history_entry)
            .collect::<Vec<_>>();
        history.extend(
            self.control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS),
        );
        history.truncate(MAX_NOTIFICATION_HISTORY);

        let selected_id = notifications
            .first()
            .map(|notification| notification.id.clone())
            .unwrap_or_default();
        let unread_count = history.iter().filter(|entry| entry_unread(entry)).count();

        self.set_visible(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, true)?;
        for property in [OPEN, POPUP_OPEN, FOCUSED, SELECTED] {
            self.mutate_control_property(
                WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                property,
                UiValue::Bool(false),
            )?;
        }
        for property in [
            INPUT_INTERACTIVE,
            INPUT_CLICKABLE,
            INPUT_HOVERABLE,
            INPUT_FOCUSABLE,
        ] {
            self.mutate_control_property(
                WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                property,
                UiValue::Bool(false),
            )?;
        }
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            NOTIFICATIONS,
            string_array_value(history),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            UNREAD_COUNT,
            UiValue::Int(unread_count as i64),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            SELECTED_NOTIFICATION_ID,
            UiValue::String(selected_id),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            FOCUSED_INDEX,
            UiValue::Int(0),
        )?;
        Ok(())
    }
}

fn string_array_value(values: Vec<String>) -> UiValue {
    UiValue::Array(values.into_iter().map(UiValue::String).collect())
}

fn entry_unread(entry: &str) -> bool {
    entry.split('|').any(|part| {
        let Some((key, value)) = part.split_once('=') else {
            return false;
        };
        matches!(key.trim(), "unread" | "new")
            && matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "true" | "1" | "yes"
            )
    })
}

fn severity_action_color(severity: WorkbenchNotificationSeverity) -> &'static str {
    match severity {
        WorkbenchNotificationSeverity::Info => "#238f98",
        WorkbenchNotificationSeverity::Success => "#2d9368",
        WorkbenchNotificationSeverity::Error => "#bf5148",
    }
}
