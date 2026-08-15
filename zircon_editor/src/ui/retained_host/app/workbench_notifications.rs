use super::*;
use std::time::Duration;

use crate::core::notifications::{
    NotificationId, NotificationSource, ToastNotification, ToastNotificationError, ToastSeverity,
};
use crate::ui::activity::{activity_progress_views, activity_toast_views};
use crate::ui::template_runtime::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    /// Retained only for the componentized host regression harness.
    #[cfg(test)]
    pub(super) fn sync_pending_play_decisions(&mut self) {
        self.sync_activity_notifications();
    }

    pub(super) fn publish_activity_toasts(&mut self, notifications: &[ToastNotification]) {
        if notifications.is_empty() {
            return;
        }

        for notification in notifications {
            match self
                .runtime
                .context()
                .notifications()
                .publish_toast(notification.clone())
            {
                Ok(()) | Err(ToastNotificationError::DuplicateNotification { .. }) => {}
                Err(error) => self.set_status_line(error.to_string()),
            }
        }
        self.sync_activity_notifications();
    }

    pub(super) fn sync_activity_notifications(&mut self) {
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return;
        }
        let pending_decisions = match self.runtime.pending_play_decision_options() {
            Ok(options) => options,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        let (toasts, progress) = {
            let context = self.runtime.context();
            let (now, toast_snapshots) = context.notifications().live_toast_snapshot();
            let toasts = activity_toast_views(&toast_snapshots, context.i18n(), now);
            let job_progress = context.jobs().progress();
            let progress_snapshots = context.notifications().progress().snapshot(&job_progress);
            let progress = activity_progress_views(&progress_snapshots, context.i18n());
            (toasts, progress)
        };
        let notification_changed = match self.workbench_window_bridge.prepare_notification_snapshot(
            &pending_decisions,
            &toasts,
            &progress,
        ) {
            Ok(changed) => changed,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        if !notification_changed && !self.pending_activity_projection_refresh {
            return;
        }
        match self.workbench_window_bridge.refresh_prepared_state_change() {
            Ok(()) => {
                self.pending_activity_projection_refresh = false;
                self.invalidate_host(HostInvalidationMask::WORKBENCH_PROJECTION);
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

pub(super) fn import_model_completed_toast() -> Option<ToastNotification> {
    activity_toast(
        "editor.import_model.completed",
        ToastSeverity::Success,
        "editor.notification.import_completed.title",
        "editor.notification.import_completed.message",
        Duration::from_millis(3_500),
    )
}

pub(super) fn import_model_failed_toast(error: &str) -> Option<ToastNotification> {
    activity_toast(
        "editor.import_model.failed",
        ToastSeverity::Error,
        "editor.notification.import_failed.title",
        ToastNotification::bounded_message(error, "The model import could not complete."),
        Duration::from_secs(7),
    )
}

fn activity_toast(
    id: &str,
    severity: ToastSeverity,
    title_key: &str,
    message_key: impl Into<String>,
    lifetime: Duration,
) -> Option<ToastNotification> {
    let id = NotificationId::parse(id).ok()?;
    let source = NotificationSource::builtin("editor.retained_host").ok()?;
    ToastNotification::new(id, source, severity, title_key, message_key, lifetime).ok()
}
