use std::{
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use crate::core::jobs::{EditorJobProgressObserver, EditorJobProgressSource, JobId};

use super::progress::{AUTOMATIC_PROGRESS_SOURCE_ID, MAX_PROGRESS_NOTIFICATIONS};
use super::{
    DecisionCenterConfig, DecisionNotificationCenter, DecisionNotificationError, NotificationId,
    NotificationSource, ProgressNotification, ProgressNotificationCenter, ToastCenterConfig,
    ToastNotification, ToastNotificationCenter, ToastNotificationError, ToastNotificationSnapshot,
};

/// Context-owned notification authority. Leaf consumers resolve immutable receipts;
/// callbacks and producer-specific mutations remain outside this service.
#[derive(Default)]
pub struct EditorNotificationService {
    decisions: OnceLock<DecisionNotificationCenter>,
    progress: OnceLock<Arc<ProgressNotificationCenter>>,
    toasts: OnceLock<ToastNotificationCenter>,
    toast_epoch: OnceLock<Instant>,
}

impl EditorNotificationService {
    pub fn decisions(&self) -> Result<&DecisionNotificationCenter, DecisionNotificationError> {
        if let Some(decisions) = self.decisions.get() {
            return Ok(decisions);
        }
        let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())?;
        let _ = self.decisions.set(center);
        Ok(self
            .decisions
            .get()
            .expect("a successful notification center initialization must publish a value"))
    }

    pub fn progress(&self) -> &ProgressNotificationCenter {
        self.progress
            .get_or_init(|| Arc::new(ProgressNotificationCenter::default()))
            .as_ref()
    }

    pub(crate) fn job_progress_observer(&self) -> Arc<dyn EditorJobProgressObserver> {
        Arc::new(EditorJobProgressNotificationObserver {
            center: Arc::clone(
                self.progress
                    .get_or_init(|| Arc::new(ProgressNotificationCenter::default())),
            ),
        })
    }

    pub fn toasts(&self) -> &ToastNotificationCenter {
        self.toasts
            .get_or_init(|| ToastNotificationCenter::new(ToastCenterConfig::default()))
    }

    /// Publishes against the context-owned monotonic epoch so leaf hosts do not invent
    /// their own expiry clocks.
    pub fn publish_toast(
        &self,
        notification: ToastNotification,
    ) -> Result<(), ToastNotificationError> {
        self.toasts().publish_at(notification, self.toast_elapsed())
    }

    pub fn toast_snapshot(&self) -> Vec<ToastNotificationSnapshot> {
        let now = self.toast_elapsed();
        self.toasts().snapshot_at(now)
    }

    pub fn live_toast_snapshot(&self) -> (Duration, Vec<ToastNotificationSnapshot>) {
        let now = self.toast_elapsed();
        (now, self.toasts().snapshot_at(now))
    }

    fn toast_elapsed(&self) -> Duration {
        self.toast_epoch.get_or_init(Instant::now).elapsed()
    }
}

struct EditorJobProgressNotificationObserver {
    center: Arc<ProgressNotificationCenter>,
}

impl EditorJobProgressObserver for EditorJobProgressNotificationObserver {
    fn job_admitted(&self, job: JobId, _source: &EditorJobProgressSource) {
        if self.center.remaining_capacity() != 0 {
            self.track_job(job);
        }
    }

    fn job_finished(&self, job: JobId, source: &EditorJobProgressSource) {
        self.center.retire_job(job);
        self.refill(source);
    }

    fn jobs_resynchronized(&self, source: &EditorJobProgressSource) {
        let _ = self.center.snapshot(source);
        self.refill(source);
    }
}

impl EditorJobProgressNotificationObserver {
    fn refill(&self, source: &EditorJobProgressSource) {
        if self.center.remaining_capacity() == 0 {
            return;
        }
        for snapshot in source.snapshot_limit(MAX_PROGRESS_NOTIFICATIONS) {
            if self.center.remaining_capacity() == 0 {
                break;
            }
            self.track_job(snapshot.id());
        }
    }

    fn track_job(&self, job: JobId) {
        let Ok(id) = NotificationId::parse(format!("editor.job.progress.{}", job.value())) else {
            return;
        };
        let Ok(source) = NotificationSource::builtin(AUTOMATIC_PROGRESS_SOURCE_ID) else {
            return;
        };
        let Ok(notification) =
            ProgressNotification::new(id, source, job, "editor.notification.job_progress.title")
        else {
            return;
        };
        let _ = self.center.publish(notification);
    }
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
