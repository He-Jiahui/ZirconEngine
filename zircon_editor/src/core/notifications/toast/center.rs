use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::time::Duration;

use crate::core::notifications::NotificationId;

use super::{ToastNotification, ToastNotificationError};

const DEFAULT_TOAST_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastCenterConfig {
    capacity: usize,
}

impl Default for ToastCenterConfig {
    fn default() -> Self {
        Self {
            capacity: DEFAULT_TOAST_CAPACITY,
        }
    }
}

impl ToastCenterConfig {
    pub fn new(capacity: usize) -> Result<Self, ToastNotificationError> {
        if capacity == 0 {
            return Err(ToastNotificationError::InvalidCapacity);
        }
        Ok(Self { capacity })
    }
    pub const fn capacity(self) -> usize {
        self.capacity
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastNotificationSnapshot {
    notification: ToastNotification,
    expires_at: Duration,
}

impl ToastNotificationSnapshot {
    fn new(notification: ToastNotification, expires_at: Duration) -> Self {
        Self {
            notification,
            expires_at,
        }
    }
    pub fn notification(&self) -> &ToastNotification {
        &self.notification
    }
    pub const fn expires_at(&self) -> Duration {
        self.expires_at
    }
}

pub struct ToastNotificationCenter {
    config: ToastCenterConfig,
    state: Mutex<ToastCenterState>,
}

#[derive(Default)]
struct ToastCenterState {
    entries: BTreeMap<NotificationId, ToastNotificationSnapshot>,
    expirations: BTreeMap<Duration, BTreeSet<NotificationId>>,
    #[cfg(test)]
    expiry_probes: usize,
}

impl ToastNotificationCenter {
    pub fn new(config: ToastCenterConfig) -> Self {
        Self {
            config,
            state: Mutex::new(ToastCenterState::default()),
        }
    }

    pub fn publish_at(
        &self,
        notification: ToastNotification,
        now: Duration,
    ) -> Result<(), ToastNotificationError> {
        let expires_at = now
            .checked_add(notification.lifetime())
            .ok_or(ToastNotificationError::InvalidLifetime)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        evict_expired(&mut state, now);
        if state.entries.contains_key(notification.id()) {
            return Err(ToastNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if state.entries.len() >= self.config.capacity() {
            return Err(ToastNotificationError::CapacityReached {
                capacity: self.config.capacity(),
            });
        }
        let notification_id = notification.id().clone();
        state.entries.insert(
            notification_id.clone(),
            ToastNotificationSnapshot::new(notification, expires_at),
        );
        state
            .expirations
            .entry(expires_at)
            .or_default()
            .insert(notification_id);
        Ok(())
    }

    pub fn snapshot_at(&self, now: Duration) -> Vec<ToastNotificationSnapshot> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        evict_expired(&mut state, now);
        state.entries.values().cloned().collect()
    }

    #[cfg(test)]
    fn expiration_group_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expirations
            .len()
    }

    #[cfg(test)]
    fn expiry_probe_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expiry_probes
    }
}

fn evict_expired(state: &mut ToastCenterState, now: Duration) {
    loop {
        #[cfg(test)]
        {
            state.expiry_probes = state.expiry_probes.saturating_add(1);
        }
        let Some((&expires_at, _)) = state.expirations.first_key_value() else {
            return;
        };
        if expires_at > now {
            return;
        }
        let (_, expired_ids) = state
            .expirations
            .pop_first()
            .expect("the first expiration group exists");
        for notification_id in expired_ids {
            state.entries.remove(&notification_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{ToastCenterConfig, ToastNotificationCenter};
    use crate::core::notifications::{
        NotificationId, NotificationSource, ToastNotification, ToastSeverity,
    };

    fn toast(index: usize, lifetime: Duration) -> ToastNotification {
        ToastNotification::new(
            NotificationId::parse(format!("editor.toast.expiry.{index}")).unwrap(),
            NotificationSource::builtin("editor10").unwrap(),
            ToastSeverity::Info,
            "editor.toast.title",
            "editor.toast.message",
            lifetime,
        )
        .unwrap()
    }

    #[test]
    fn optimization_wave_20260824c_editor10_expiry_index_preserves_deadline_groups() {
        let center = ToastNotificationCenter::new(ToastCenterConfig::new(3).unwrap());
        center
            .publish_at(toast(1, Duration::from_secs(5)), Duration::ZERO)
            .unwrap();
        center
            .publish_at(toast(2, Duration::from_secs(7)), Duration::ZERO)
            .unwrap();
        center
            .publish_at(toast(3, Duration::from_secs(7)), Duration::ZERO)
            .unwrap();

        let snapshots = center.snapshot_at(Duration::from_secs(5));
        assert_eq!(snapshots.len(), 2);
        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| snapshot.notification().id().as_str())
                .collect::<Vec<_>>(),
            ["editor.toast.expiry.2", "editor.toast.expiry.3"]
        );
        assert_eq!(center.expiration_group_count(), 1);

        assert!(center.snapshot_at(Duration::from_secs(7)).is_empty());
        assert_eq!(center.expiration_group_count(), 0);
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260824c_editor10_expiry_index_evidence() {
        const TOASTS: usize = 10_000;
        const MAX_ELAPSED_NS: u128 = 5_000_000_000;

        let center = ToastNotificationCenter::new(ToastCenterConfig::new(TOASTS).unwrap());
        let started = Instant::now();
        for index in 0..TOASTS {
            center
                .publish_at(toast(index, Duration::from_secs(3_600)), Duration::ZERO)
                .unwrap();
        }
        assert_eq!(center.snapshot_at(Duration::ZERO).len(), TOASTS);
        let elapsed_ns = started.elapsed().as_nanos();
        let optimized_expiry_probes = center.expiry_probe_count();
        let legacy_expiry_probes = TOASTS.saturating_mul(TOASTS.saturating_sub(1)) / 2 + TOASTS;
        let probe_reduction_bps = legacy_expiry_probes
            .saturating_sub(optimized_expiry_probes)
            .saturating_mul(10_000)
            / legacy_expiry_probes;

        println!(
            "EDITOR_TOAST_EXPIRY_BENCH_V1 toasts={TOASTS} legacy_expiry_probes={legacy_expiry_probes} optimized_expiry_probes={optimized_expiry_probes} probe_reduction_bps={probe_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
        );

        assert!(optimized_expiry_probes <= TOASTS + 1);
        assert!(probe_reduction_bps >= 9_998);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);
    }
}
