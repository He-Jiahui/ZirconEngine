use std::{
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use crate::core::jobs::{EditorJobProgressObserver, EditorJobProgressSource, JobId};

use super::progress::{AUTOMATIC_PROGRESS_SOURCE_ID, MAX_PROGRESS_NOTIFICATIONS};
use super::{
    DecisionCenterConfig, DecisionNotificationCenter, DecisionNotificationError, NotificationId,
    NotificationSource, ProgressNotification, ProgressNotificationCenter, ToastCenterConfig,
    ToastDeliveryQueue, ToastDeliverySnapshot, ToastNotification, ToastNotificationError,
};

/// Context-owned notification authority. Leaf consumers resolve immutable receipts;
/// callbacks and producer-specific mutations remain outside this service.
#[derive(Default)]
pub struct EditorNotificationService {
    decisions: OnceLock<DecisionNotificationCenter>,
    progress: OnceLock<Arc<ProgressNotificationCenter>>,
    toasts: OnceLock<ToastDeliveryQueue>,
    toast_epoch: OnceLock<Instant>,
}

impl EditorNotificationService {
    /// Pending decisions, live toasts and active job progress; reading does not create centers.
    pub fn active_count(&self) -> usize {
        let decisions = self.decisions.get().map_or(0, |center| center.pending_count());
        let progress = self.progress.get().map_or(0, |center| center.active_count());
        let toasts = self.toasts.get().map_or(0, |queue| {
            queue.snapshot_at(self.toast_elapsed()).len()
        });
        decisions.saturating_add(progress).saturating_add(toasts)
    }

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

    pub fn toasts(&self) -> &ToastDeliveryQueue {
        self.toasts
            .get_or_init(|| ToastDeliveryQueue::new(ToastCenterConfig::default()))
    }

    /// Queues in publication order without a deadline. Only a confirmed native presentation
    /// starts lifetime, against this context's monotonic epoch.
    pub fn publish_toast(
        &self,
        notification: ToastNotification,
    ) -> Result<(), ToastNotificationError> {
        self.toasts().publish_at(notification, self.toast_elapsed())
    }

    pub fn toast_snapshot(&self) -> Vec<ToastDeliverySnapshot> {
        let now = self.toast_elapsed();
        self.toasts().snapshot_at(now)
    }

    pub fn live_toast_snapshot(&self) -> (Duration, Vec<ToastDeliverySnapshot>) {
        let now = self.toast_elapsed();
        (now, self.toasts().snapshot_at(now))
    }

    pub(crate) fn acknowledge_toast_presented(
        &self,
        id: &str,
        token: u64,
    ) -> Result<bool, ToastNotificationError> {
        let id = NotificationId::parse(id)?;
        self.toasts()
            .acknowledge_at(&id, token, self.toast_elapsed())
    }

    pub(crate) fn dismiss_toast(&self, id: &str, token: u64) -> bool {
        NotificationId::parse(id)
            .ok()
            .is_some_and(|id| self.toasts().dismiss(&id, token))
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
