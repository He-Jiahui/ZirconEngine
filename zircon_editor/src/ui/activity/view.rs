use std::time::Duration;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

use crate::core::i18n::EditorI18nService;
use crate::core::jobs::JobId;
use crate::core::logging::{LogJump, LogRecord, LogSeverity, LogSource};
use crate::core::notifications::{
    present_progress, present_toast, LocalizedProgressNotification, LocalizedToastNotification,
    ProgressNotificationSnapshot, ToastNotificationSnapshot, ToastSeverity,
};

use super::slot::ActivityDrawerSlotPreference;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityToastView {
    id: String,
    title: String,
    message: String,
    severity: ToastSeverity,
    expires_at: Duration,
    remaining_lifetime: Duration,
}

impl ActivityToastView {
    pub(crate) fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: ToastSeverity,
        expires_at: Duration,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            severity,
            expires_at,
            remaining_lifetime: expires_at,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) const fn severity(&self) -> ToastSeverity {
        self.severity
    }

    pub(crate) const fn expires_at(&self) -> Duration {
        self.expires_at
    }

    pub(crate) const fn remaining_lifetime(&self) -> Duration {
        self.remaining_lifetime
    }
}

pub(crate) fn activity_toast_views(
    snapshots: &[ToastNotificationSnapshot],
    i18n: &EditorI18nService,
    now: Duration,
) -> Vec<ActivityToastView> {
    let mut views = Vec::with_capacity(snapshots.len());
    views.extend(
        snapshots
            .iter()
            .map(|snapshot| activity_toast_view(&present_toast(snapshot, i18n), now)),
    );
    views
}

fn activity_toast_view(
    notification: &LocalizedToastNotification,
    now: Duration,
) -> ActivityToastView {
    let mut view = ActivityToastView::new(
        notification.id().as_str(),
        notification.title(),
        notification.message(),
        notification.severity(),
        notification.expires_at(),
    );
    view.remaining_lifetime = notification.expires_at().saturating_sub(now);
    view
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityProgressView {
    id: String,
    job_id: JobId,
    title: String,
    detail: String,
    percent: Option<u8>,
}

impl ActivityProgressView {
    pub(crate) fn new(
        id: impl Into<String>,
        job_id: JobId,
        title: impl Into<String>,
        detail: impl Into<String>,
        percent: Option<u8>,
    ) -> Self {
        Self {
            id: id.into(),
            job_id,
            title: title.into(),
            detail: detail.into(),
            percent,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) const fn job_id(&self) -> JobId {
        self.job_id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn detail(&self) -> &str {
        &self.detail
    }

    pub(crate) const fn percent(&self) -> Option<u8> {
        self.percent
    }
}

pub(crate) fn activity_progress_views(
    snapshots: &[ProgressNotificationSnapshot],
    i18n: &EditorI18nService,
) -> Vec<ActivityProgressView> {
    let mut views = Vec::with_capacity(snapshots.len());
    views.extend(
        snapshots
            .iter()
            .map(|snapshot| activity_progress_view(&present_progress(snapshot, i18n))),
    );
    views
}

fn activity_progress_view(notification: &LocalizedProgressNotification) -> ActivityProgressView {
    let job = notification.job();
    let detail = job
        .progress()
        .map(|progress| progress.message().trim())
        .filter(|message| !message.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| job.label().to_string());
    let percent = job.progress().and_then(|progress| {
        (progress.total() != 0).then(|| {
            ((u64::from(progress.completed()) * 100) / u64::from(progress.total())).min(100) as u8
        })
    });

    ActivityProgressView::new(
        notification.id().as_str(),
        job.id(),
        notification.title(),
        detail,
        percent,
    )
}

/// Read-only activity projection over an immutable log record.
///
/// Filtering and retention remain owned by `EditorLogService`; the activity layer only exposes
/// the typed record fields that a diagnostics surface needs to render and dispatch a jump.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityLogView {
    record: LogRecord,
}

impl ActivityLogView {
    fn new(record: LogRecord) -> Self {
        Self { record }
    }

    pub(crate) const fn sequence(&self) -> u64 {
        self.record.sequence()
    }

    pub(crate) fn source(&self) -> &LogSource {
        self.record.entry().source()
    }

    pub(crate) fn severity(&self) -> LogSeverity {
        self.record.entry().severity()
    }

    pub(crate) fn message(&self) -> &str {
        self.record.entry().message()
    }

    pub(crate) fn timestamp_frame(&self) -> u64 {
        self.record.entry().timestamp_frame()
    }

    pub(crate) fn jump(&self) -> Option<&LogJump> {
        self.record.entry().jump()
    }
}

/// Builds a read-only Activity view over the already-filtered logging snapshot.
pub(crate) fn activity_log_views(records: &[LogRecord]) -> Vec<ActivityLogView> {
    let mut views = Vec::with_capacity(records.len());
    views.extend(records.iter().cloned().map(ActivityLogView::new));
    views
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityViewDescriptor {
    pub view_id: String,
    pub title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub supports_document_host: bool,
    pub supports_floating_window: bool,
    pub default_drawer: Option<ActivityDrawerSlotPreference>,
    pub reflection_root: UiNodePath,
}

impl ActivityViewDescriptor {
    pub fn new(
        view_id: impl Into<String>,
        title: impl Into<String>,
        icon_key: impl Into<String>,
    ) -> Self {
        let view_id = view_id.into();
        Self {
            reflection_root: UiNodePath::new(format!("editor/views/{view_id}")),
            view_id,
            title: title.into(),
            icon_key: icon_key.into(),
            multi_instance: false,
            supports_document_host: true,
            supports_floating_window: true,
            default_drawer: None,
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_supports_document_host(mut self, supports: bool) -> Self {
        self.supports_document_host = supports;
        self
    }

    pub fn with_supports_floating_window(mut self, supports: bool) -> Self {
        self.supports_floating_window = supports;
        self
    }

    pub fn with_default_drawer(mut self, slot: ActivityDrawerSlotPreference) -> Self {
        self.default_drawer = Some(slot);
        self
    }

    pub fn with_reflection_root(mut self, root: UiNodePath) -> Self {
        self.reflection_root = root;
        self
    }
}

#[cfg(test)]
#[path = "tests/view.rs"]
mod tests;
