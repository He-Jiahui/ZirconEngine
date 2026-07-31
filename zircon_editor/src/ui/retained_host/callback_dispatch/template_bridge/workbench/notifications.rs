use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::host::play_pending_decision::PlayPendingDecisionOption;
use crate::ui::retained_host::workbench_notifications::{
    WorkbenchNotification, WorkbenchNotificationSeverity,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

#[path = "notifications/history.rs"]
mod history;

use history::RetainedNotificationHistory;

pub(crate) const WORKBENCH_TOAST_CONTROL_ID: &str = "WorkbenchToast";
pub(crate) const WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID: &str = "WorkbenchNotificationCenter";

const MAX_NOTIFICATION_HISTORY: usize = 64;
const MAX_TOAST_QUEUE: usize = MAX_NOTIFICATION_HISTORY;
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
const NOTIFICATION_GENERATION: &str = "notification_generation";
const OVERFLOW_COUNT: &str = "overflow_count";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const FOCUSED_INDEX: &str = "focused_index";
const INPUT_INTERACTIVE: &str = "input_interactive";
const INPUT_CLICKABLE: &str = "input_clickable";
const INPUT_HOVERABLE: &str = "input_hoverable";
const INPUT_FOCUSABLE: &str = "input_focusable";
const CLOSE_ON_BACKDROP_CLICK: &str = "close_on_backdrop_click";
const DISABLE_ESCAPE_KEY_DOWN: &str = "disable_escape_key_down";
const PLAY_PENDING_DECISION_KIND: &str = "play_pending_decision";

#[derive(Clone, Copy, Default)]
struct NotificationCounters {
    generation: i64,
    overflow_count: i64,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn sync_pending_play_decision_options(
        &mut self,
        options: &[PlayPendingDecisionOption],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID) {
            return Ok(false);
        }

        let existing =
            self.control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS);
        let counters = self.notification_counters();
        let retained = RetainedNotificationHistory::merge(
            options.iter().map(pending_play_decision_history_entry),
            existing
                .iter()
                .filter(|entry| !is_pending_play_decision_entry(entry))
                .cloned(),
            MAX_NOTIFICATION_HISTORY,
            counters.overflow_count,
        );

        let decision_open = !options.is_empty();
        let selected_id = options
            .first()
            .map(|option| option.selection_id().to_string())
            .unwrap_or_default();
        let history_changed = existing != retained.entries;
        let notification_generation =
            next_notification_generation(counters.generation, history_changed);
        let changed = history_changed
            || self.control_bool(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, OPEN) != decision_open
            || self.control_bool(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, POPUP_OPEN)
                != decision_open
            || self.control_bool(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, INPUT_INTERACTIVE)
                != decision_open
            || self
                .control_string(
                    WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                    SELECTED_NOTIFICATION_ID,
                )
                .as_deref()
                != Some(selected_id.as_str());
        if !changed {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, true)?;
        for property in [
            OPEN,
            POPUP_OPEN,
            INPUT_INTERACTIVE,
            INPUT_CLICKABLE,
            INPUT_HOVERABLE,
            INPUT_FOCUSABLE,
        ] {
            self.mutate_control_property(
                WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                property,
                UiValue::Bool(decision_open),
            )?;
        }
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            CLOSE_ON_BACKDROP_CLICK,
            UiValue::Bool(!decision_open),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            DISABLE_ESCAPE_KEY_DOWN,
            UiValue::Bool(decision_open),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            NOTIFICATIONS,
            string_array_value(retained.entries),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            UNREAD_COUNT,
            UiValue::Int(retained.unread_count),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            OVERFLOW_COUNT,
            UiValue::Int(retained.overflow_count),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            NOTIFICATION_GENERATION,
            UiValue::Int(notification_generation),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            SELECTED_NOTIFICATION_ID,
            UiValue::String(selected_id),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            FOCUSED_INDEX,
            UiValue::Int(if decision_open { 0 } else { -1 }),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn is_pending_play_decision_option(
        &self,
        control_id: &str,
        option_id: &str,
    ) -> bool {
        control_id == WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID
            && self
                .control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS)
                .iter()
                .any(|entry| entry_id(entry) == option_id && is_pending_play_decision_entry(entry))
    }

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
            .take(MAX_TOAST_QUEUE)
            .map(WorkbenchNotification::toast_queue_entry)
            .collect::<Vec<_>>();
        let queue_length = queue.len() as i64;
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
            UiValue::Int(queue_length),
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
        let counters = self.notification_counters();
        let retained = RetainedNotificationHistory::merge(
            notifications
                .iter()
                .map(WorkbenchNotification::history_entry),
            self.control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS),
            MAX_NOTIFICATION_HISTORY,
            counters.overflow_count,
        );

        let selected_id = notifications
            .first()
            .map(|notification| notification.id.clone())
            .unwrap_or_default();
        let notification_generation = next_notification_generation(counters.generation, true);
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
            string_array_value(retained.entries),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            UNREAD_COUNT,
            UiValue::Int(retained.unread_count),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            OVERFLOW_COUNT,
            UiValue::Int(retained.overflow_count),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            NOTIFICATION_GENERATION,
            UiValue::Int(notification_generation),
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

    fn notification_counters(&self) -> NotificationCounters {
        self.template_surface
            .surface
            .tree
            .nodes
            .values()
            .find_map(|node| {
                node.template_metadata
                    .as_ref()
                    .filter(|metadata| {
                        metadata.control_id.as_deref()
                            == Some(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID)
                    })
                    .map(|metadata| NotificationCounters {
                        generation: non_negative_integer(
                            metadata.attributes.get(NOTIFICATION_GENERATION),
                        ),
                        overflow_count: non_negative_integer(
                            metadata.attributes.get(OVERFLOW_COUNT),
                        ),
                    })
            })
            .unwrap_or_default()
    }
}

fn non_negative_integer(value: Option<&toml::Value>) -> i64 {
    value.and_then(toml::Value::as_integer).unwrap_or(0).max(0)
}

fn next_notification_generation(generation: i64, history_changed: bool) -> i64 {
    if history_changed {
        generation.saturating_add(1)
    } else {
        generation
    }
}

fn string_array_value(values: Vec<String>) -> UiValue {
    UiValue::Array(values.into_iter().map(UiValue::String).collect())
}

fn pending_play_decision_history_entry(option: &PlayPendingDecisionOption) -> String {
    format!(
        "{}|title={}|message={}|severity=info|unread=true|kind={PLAY_PENDING_DECISION_KIND}",
        option.selection_id(),
        option.title(),
        option.message(),
    )
}

fn is_pending_play_decision_entry(entry: &str) -> bool {
    entry
        .split('|')
        .any(|part| part.trim().strip_prefix("kind=") == Some(PLAY_PENDING_DECISION_KIND))
}

fn entry_id(entry: &str) -> &str {
    entry.split('|').next().unwrap_or_default().trim()
}

pub(super) fn entry_unread(entry: &str) -> bool {
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
