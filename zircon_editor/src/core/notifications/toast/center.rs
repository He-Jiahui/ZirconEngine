//! 保留有限的短时提示并按截止分组回收；调用端须使用同一单调时间基准发布和读取，提示过期后才释放重复身份和容量。
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

    /// 发布和快照调用须共用单调纪元；先回收已过期项，再核对存活身份与容量。
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
#[path = "tests/center.rs"]
mod tests;
