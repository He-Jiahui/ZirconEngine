use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

use super::{
    DecisionCenterConfig, DecisionNotificationCenter, DecisionNotificationError,
    ProgressNotificationCenter, ToastCenterConfig, ToastNotification, ToastNotificationCenter,
    ToastNotificationError, ToastNotificationSnapshot,
};

/// Context-owned notification authority. Leaf consumers resolve immutable receipts;
/// callbacks and producer-specific mutations remain outside this service.
#[derive(Default)]
pub struct EditorNotificationService {
    decisions: OnceLock<DecisionNotificationCenter>,
    progress: OnceLock<ProgressNotificationCenter>,
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
            .get_or_init(ProgressNotificationCenter::default)
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
