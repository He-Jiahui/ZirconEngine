use crate::core::editor_event::{EditorEventEffect, EditorEventRecord};

const DEFAULT_TOAST_DURATION_MS: i64 = 3_500;
const IMPORT_TOAST_DURATION_MS: i64 = 4_000;
const ERROR_TOAST_DURATION_MS: i64 = 7_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchNotificationSeverity {
    Info,
    Success,
    Error,
}

impl WorkbenchNotificationSeverity {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Error => "error",
        }
    }

    pub(crate) fn icon(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "check-circle",
            Self::Error => "alert-circle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchNotification {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) severity: WorkbenchNotificationSeverity,
    pub(crate) unread: bool,
    pub(crate) action_label: Option<String>,
    pub(crate) auto_hide_duration_ms: i64,
}

impl WorkbenchNotification {
    pub(crate) fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: WorkbenchNotificationSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            severity,
            unread: true,
            action_label: None,
            auto_hide_duration_ms: DEFAULT_TOAST_DURATION_MS,
        }
    }

    pub(crate) fn with_action_label(mut self, action_label: impl Into<String>) -> Self {
        let action_label = action_label.into();
        if !action_label.trim().is_empty() {
            self.action_label = Some(action_label);
        }
        self
    }

    pub(crate) fn with_duration_ms(mut self, duration_ms: i64) -> Self {
        if duration_ms > 0 {
            self.auto_hide_duration_ms = duration_ms;
        }
        self
    }

    pub(crate) fn toast_message(&self) -> &str {
        if self.message.trim().is_empty() {
            self.title.as_str()
        } else {
            self.message.as_str()
        }
    }

    pub(crate) fn toast_queue_entry(&self) -> String {
        let mut entry = format!(
            "{}|message={}|title={}|severity={}|auto_hide_duration_ms={}",
            pipe_value(&self.id),
            pipe_value(self.toast_message()),
            pipe_value(&self.title),
            self.severity.as_str(),
            self.auto_hide_duration_ms
        );
        if let Some(action_label) = self.action_label.as_deref() {
            entry.push_str("|action_label=");
            entry.push_str(&pipe_value(action_label));
        }
        entry
    }

    pub(crate) fn history_entry(&self) -> String {
        let mut entry = format!(
            "{}|title={}|message={}|severity={}|unread={}",
            pipe_value(&self.id),
            pipe_value(&self.title),
            pipe_value(&self.message),
            self.severity.as_str(),
            self.unread
        );
        if let Some(action_label) = self.action_label.as_deref() {
            entry.push_str("|action_label=");
            entry.push_str(&pipe_value(action_label));
        }
        entry
    }
}

pub(crate) fn workbench_notifications_for_record(
    record: &EditorEventRecord,
) -> Vec<WorkbenchNotification> {
    if let Some(error) = record
        .result
        .error
        .as_deref()
        .map(str::trim)
        .filter(|error| !error.is_empty())
    {
        return vec![workbench_record_error_notification(record, error)];
    }

    let mut notifications = Vec::new();
    for effect in &record.effects {
        let notification = match effect {
            EditorEventEffect::ProjectOpenRequested => Some(
                WorkbenchNotification::new(
                    notification_id(record, "project-open"),
                    "Project opened",
                    "Project workspace is ready.",
                    WorkbenchNotificationSeverity::Success,
                )
                .with_duration_ms(DEFAULT_TOAST_DURATION_MS),
            ),
            EditorEventEffect::ProjectSaveRequested => Some(
                WorkbenchNotification::new(
                    notification_id(record, "project-save"),
                    "Project saved",
                    "Project state was written to disk.",
                    WorkbenchNotificationSeverity::Success,
                )
                .with_duration_ms(DEFAULT_TOAST_DURATION_MS),
            ),
            EditorEventEffect::ImportModelRequested => Some(
                WorkbenchNotification::new(
                    notification_id(record, "import-model"),
                    "Import model",
                    "Choose a model file to import into the active project.",
                    WorkbenchNotificationSeverity::Info,
                )
                .with_action_label("Import")
                .with_duration_ms(IMPORT_TOAST_DURATION_MS),
            ),
            _ => None,
        };
        if let Some(notification) = notification {
            notifications.push(notification);
        }
    }
    notifications
}

pub(crate) fn workbench_import_model_completed_notification() -> WorkbenchNotification {
    WorkbenchNotification::new(
        "import-model-complete",
        "Model imported",
        "Model asset was staged and added to the scene.",
        WorkbenchNotificationSeverity::Success,
    )
    .with_duration_ms(DEFAULT_TOAST_DURATION_MS)
}

pub(crate) fn workbench_import_model_failed_notification(error: &str) -> WorkbenchNotification {
    WorkbenchNotification::new(
        "import-model-failed",
        "Model import failed",
        non_empty_error(error),
        WorkbenchNotificationSeverity::Error,
    )
    .with_action_label("Review")
    .with_duration_ms(ERROR_TOAST_DURATION_MS)
}

pub(crate) fn workbench_dispatch_error_notification(error: &str) -> WorkbenchNotification {
    WorkbenchNotification::new(
        "editor-command-failed",
        "Command failed",
        non_empty_error(error),
        WorkbenchNotificationSeverity::Error,
    )
    .with_duration_ms(ERROR_TOAST_DURATION_MS)
}

fn workbench_record_error_notification(
    record: &EditorEventRecord,
    error: &str,
) -> WorkbenchNotification {
    WorkbenchNotification::new(
        notification_id(record, "error"),
        "Command failed",
        non_empty_error(error),
        WorkbenchNotificationSeverity::Error,
    )
    .with_duration_ms(ERROR_TOAST_DURATION_MS)
}

fn notification_id(record: &EditorEventRecord, suffix: &str) -> String {
    format!("editor-event-{}-{suffix}", record.sequence.0)
}

fn non_empty_error(error: &str) -> String {
    let error = error.trim();
    if error.is_empty() {
        "The editor command could not complete.".to_string()
    } else {
        error.to_string()
    }
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
