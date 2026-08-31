use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use crate::core::notifications::ToastSeverity;
use crate::ui::activity::{ActivityDecisionOption, ActivityProgressView, ActivityToastView};

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
const DECISION_KIND: &str = "decision";

#[derive(Clone, Copy, Default)]
struct NotificationCounters {
    generation: i64,
    overflow_count: i64,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    /// Projects one current core snapshot. The retained bridge never uses the previous control
    /// contents as notification authority, so expired Toasts and Progress rows disappear as soon
    /// as their core snapshots do.
    pub(crate) fn sync_notification_snapshot(
        &mut self,
        pending_decisions: &[ActivityDecisionOption],
        toasts: &[ActivityToastView],
        progress: &[ActivityProgressView],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let changed = self.prepare_notification_snapshot(pending_decisions, toasts, progress)?;
        if changed {
            self.refresh_prepared_state_change()?;
        }
        Ok(changed)
    }

    pub(crate) fn prepare_notification_snapshot(
        &mut self,
        pending_decisions: &[ActivityDecisionOption],
        toasts: &[ActivityToastView],
        progress: &[ActivityProgressView],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let mut changed = false;
        if self.has_control(WORKBENCH_TOAST_CONTROL_ID) {
            changed |= self.sync_toast_queue(toasts)?;
        }
        if self.has_control(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID) {
            changed |= self.sync_notification_projection(pending_decisions, toasts, progress)?;
        }
        Ok(changed)
    }

    pub(crate) fn is_pending_activity_decision_option(
        &self,
        control_id: &str,
        option_id: &str,
    ) -> bool {
        control_id == WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID
            && self
                .control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS)
                .iter()
                .any(|entry| entry_id(entry) == option_id && is_pending_decision_entry(entry))
    }

    fn sync_toast_queue(
        &mut self,
        notifications: &[ActivityToastView],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let queue = notifications
            .iter()
            .take(MAX_TOAST_QUEUE)
            .map(activity_toast_queue_entry)
            .collect::<Vec<_>>();
        let queue_length = queue.len() as i64;
        let visible = !notifications.is_empty();
        let (id, message, severity, duration_ms) = notifications
            .first()
            .map(|notification| {
                (
                    notification.id().to_string(),
                    notification.message().to_string(),
                    toast_severity_name(notification.severity()).to_string(),
                    duration_millis(notification.remaining_lifetime()),
                )
            })
            .unwrap_or_default();
        let expired_id = if visible {
            String::new()
        } else {
            self.control_string(WORKBENCH_TOAST_CONTROL_ID, CURRENT_TOAST_ID)
                .unwrap_or_default()
        };
        let expected_visibility = if visible { "visible" } else { "collapsed" };
        let existing_queue = self.control_string_array(WORKBENCH_TOAST_CONTROL_ID, TOAST_QUEUE);
        let changed = self
            .control_string(WORKBENCH_TOAST_CONTROL_ID, "visibility")
            .as_deref()
            != Some(expected_visibility)
            || self.control_bool(WORKBENCH_TOAST_CONTROL_ID, OPEN) != visible
            || self.control_bool(WORKBENCH_TOAST_CONTROL_ID, POPUP_OPEN) != visible
            || self
                .control_string(WORKBENCH_TOAST_CONTROL_ID, TEXT)
                .as_deref()
                != Some(message.as_str())
            || self
                .control_string(WORKBENCH_TOAST_CONTROL_ID, CURRENT_TOAST_ID)
                .as_deref()
                != Some(id.as_str())
            || !toast_queue_semantically_equal(&existing_queue, &queue);
        if !changed {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_TOAST_CONTROL_ID, visible)?;
        for property in [OPEN, POPUP_OPEN] {
            self.mutate_control_property(
                WORKBENCH_TOAST_CONTROL_ID,
                property,
                UiValue::Bool(visible),
            )?;
        }
        self.mutate_control_property(WORKBENCH_TOAST_CONTROL_ID, FOCUSED, UiValue::Bool(false))?;
        self.mutate_control_property(WORKBENCH_TOAST_CONTROL_ID, SELECTED, UiValue::Bool(false))?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            TEXT,
            UiValue::String(message.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            VALUE,
            UiValue::String(id.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            VALUE_TEXT,
            UiValue::String(message),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            SEVERITY,
            UiValue::String(severity.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            COLOR,
            UiValue::String(severity.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ICON,
            UiValue::String(
                toast_severity_icon(notifications.first().map(ActivityToastView::severity))
                    .to_string(),
            ),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ACTION_LABEL,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            ACTION_COLOR,
            UiValue::String(
                toast_severity_color(notifications.first().map(ActivityToastView::severity))
                    .to_string(),
            ),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            TOAST_QUEUE,
            string_array_value(queue),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            CURRENT_TOAST_ID,
            UiValue::String(id),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            EXPIRED_TOAST_ID,
            UiValue::String(expired_id),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            QUEUE_LENGTH,
            UiValue::Int(queue_length),
        )?;
        self.mutate_control_property(
            WORKBENCH_TOAST_CONTROL_ID,
            AUTO_HIDE_DURATION_MS,
            UiValue::Int(duration_ms),
        )?;
        Ok(true)
    }

    fn sync_notification_projection(
        &mut self,
        pending_decisions: &[ActivityDecisionOption],
        toasts: &[ActivityToastView],
        progress: &[ActivityProgressView],
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let counters = self.notification_counters();
        let existing =
            self.control_string_array(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, NOTIFICATIONS);
        let notification_count = pending_decisions
            .len()
            .saturating_add(progress.len())
            .saturating_add(toasts.len());
        let entries = pending_decisions
            .iter()
            .map(activity_decision_history_entry)
            .chain(progress.iter().map(activity_progress_history_entry))
            .chain(toasts.iter().map(activity_toast_history_entry))
            .take(MAX_NOTIFICATION_HISTORY)
            .collect::<Vec<_>>();
        let overflow_count = notification_count.saturating_sub(MAX_NOTIFICATION_HISTORY) as i64;
        let unread_count = entries.iter().filter(|entry| entry_unread(entry)).count() as i64;
        let selected_id = pending_decisions
            .first()
            .map(|option| option.selection_id().as_str().to_string())
            .or_else(|| {
                progress
                    .first()
                    .map(|notification| notification.id().to_string())
            })
            .or_else(|| {
                toasts
                    .first()
                    .map(|notification| notification.id().to_string())
            })
            .unwrap_or_default();
        let history_changed = existing != entries;
        let notification_generation =
            next_notification_generation(counters.generation, history_changed);
        let visible = !entries.is_empty();
        let decision_open = !pending_decisions.is_empty();
        let changed = history_changed
            || counters.overflow_count != overflow_count
            || self
                .control_string(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, "visibility")
                .as_deref()
                != Some(if visible { "visible" } else { "collapsed" })
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
                != Some(selected_id.as_str())
            || self.control_bool(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, FOCUSED)
            || self.control_bool(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, SELECTED);
        if !changed {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, visible)?;
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
        for property in [FOCUSED, SELECTED] {
            self.mutate_control_property(
                WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                property,
                UiValue::Bool(false),
            )?;
        }
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            NOTIFICATIONS,
            string_array_value(entries),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            UNREAD_COUNT,
            UiValue::Int(unread_count),
        )?;
        self.mutate_control_property(
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            OVERFLOW_COUNT,
            UiValue::Int(overflow_count),
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
            UiValue::Int(if visible { 0 } else { -1 }),
        )?;
        Ok(true)
    }

    fn notification_counters(&self) -> NotificationCounters {
        NotificationCounters {
            generation: self
                .control_integer(
                    WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
                    NOTIFICATION_GENERATION,
                )
                .unwrap_or(0)
                .max(0),
            overflow_count: self
                .control_integer(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, OVERFLOW_COUNT)
                .unwrap_or(0)
                .max(0),
        }
    }
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

fn activity_decision_history_entry(option: &ActivityDecisionOption) -> String {
    format!(
        "{}|title={}|message={}|severity=info|unread=true|kind={DECISION_KIND}",
        option.selection_id().as_str(),
        option.title(),
        option.message(),
    )
}

fn is_pending_decision_entry(entry: &str) -> bool {
    entry_kind(entry) == Some(DECISION_KIND)
}

fn entry_kind(entry: &str) -> Option<&str> {
    entry
        .split('|')
        .find_map(|part| part.trim().strip_prefix("kind="))
}

fn entry_id(entry: &str) -> &str {
    entry.split('|').next().unwrap_or_default().trim()
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

fn activity_toast_queue_entry(notification: &ActivityToastView) -> String {
    format!(
        "{}|message={}|title={}|severity={}|auto_hide_duration_ms={}",
        pipe_value(notification.id()),
        pipe_value(notification.message()),
        pipe_value(notification.title()),
        toast_severity_name(notification.severity()),
        duration_millis(notification.remaining_lifetime()),
    )
}

fn toast_queue_semantically_equal(previous: &[String], next: &[String]) -> bool {
    previous.len() == next.len()
        && previous
            .iter()
            .zip(next)
            .all(|(previous, next)| toast_queue_entry_semantically_equal(previous, next))
}

fn toast_queue_entry_semantically_equal(previous: &str, next: &str) -> bool {
    let mut previous = previous
        .split('|')
        .filter(|field| !volatile_toast_queue_field(field));
    let mut next = next
        .split('|')
        .filter(|field| !volatile_toast_queue_field(field));
    loop {
        match (previous.next(), next.next()) {
            (Some(previous), Some(next)) if previous == next => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn volatile_toast_queue_field(field: &str) -> bool {
    field.split_once('=').is_some_and(|(key, _)| {
        matches!(
            key.trim(),
            "duration" | "duration_ms" | "auto_hide_duration_ms" | "autoHideDuration"
        )
    })
}

fn activity_toast_history_entry(notification: &ActivityToastView) -> String {
    format!(
        "{}|title={}|message={}|severity={}|unread=true|kind=toast",
        pipe_value(notification.id()),
        pipe_value(notification.title()),
        pipe_value(notification.message()),
        toast_severity_name(notification.severity()),
    )
}

fn activity_progress_history_entry(notification: &ActivityProgressView) -> String {
    let percent = notification
        .percent()
        .map(|percent| percent.to_string())
        .unwrap_or_else(|| "indeterminate".to_string());
    format!(
        "{}|title={}|message={}|severity=info|unread=false|kind=progress|job_id={}|percent={percent}",
        pipe_value(notification.id()),
        pipe_value(notification.title()),
        pipe_value(notification.detail()),
        notification.job_id().value(),
    )
}

fn pipe_value(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '|' | '=' | '\n' | '\r' | '\t' => ' ',
            _ => ch,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn duration_millis(duration: std::time::Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

fn toast_severity_name(severity: ToastSeverity) -> &'static str {
    match severity {
        ToastSeverity::Info => "info",
        ToastSeverity::Success => "success",
        ToastSeverity::Warning => "warning",
        ToastSeverity::Error => "error",
    }
}

fn toast_severity_icon(severity: Option<ToastSeverity>) -> &'static str {
    match severity {
        Some(ToastSeverity::Success) => "check-circle",
        Some(ToastSeverity::Warning) => "alert-triangle",
        Some(ToastSeverity::Error) => "alert-circle",
        Some(ToastSeverity::Info) | None => "info",
    }
}

fn toast_severity_color(severity: Option<ToastSeverity>) -> &'static str {
    match severity {
        Some(ToastSeverity::Info) | None => "#238f98",
        Some(ToastSeverity::Success) => "#2d9368",
        Some(ToastSeverity::Warning) => "#b87918",
        Some(ToastSeverity::Error) => "#bf5148",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn notification_history_bounds_entry_formatting_before_allocation() {
        let source = include_str!("notifications.rs");
        let projection = source
            .split("fn sync_notification_projection")
            .nth(1)
            .and_then(|source| source.split("fn notification_counters").next())
            .expect("notification projection source must remain isolated");
        let capped_iteration = projection
            .find(".take(MAX_NOTIFICATION_HISTORY)")
            .expect("history input must be capped before entry formatting");
        let formatted_entries = projection
            .find(".collect::<Vec<_>>()")
            .expect("history entries must be materialized once after the cap");

        assert!(capped_iteration < formatted_entries);
        assert!(!projection.contains("candidate_entries"));
        assert!(projection.contains("pending_decisions.len()"));
        assert!(projection.contains("progress.len()"));
        assert!(projection.contains("toasts.len()"));
    }
}
