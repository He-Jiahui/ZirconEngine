use std::collections::BTreeMap;
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
    entries: Mutex<BTreeMap<NotificationId, ToastNotificationSnapshot>>,
}

impl ToastNotificationCenter {
    pub fn new(config: ToastCenterConfig) -> Self {
        Self {
            config,
            entries: Mutex::new(BTreeMap::new()),
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
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.retain(|_, snapshot| snapshot.expires_at() > now);
        if entries.contains_key(notification.id()) {
            return Err(ToastNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if entries.len() >= self.config.capacity() {
            return Err(ToastNotificationError::CapacityReached {
                capacity: self.config.capacity(),
            });
        }
        entries.insert(
            notification.id().clone(),
            ToastNotificationSnapshot::new(notification, expires_at),
        );
        Ok(())
    }

    pub fn snapshot_at(&self, now: Duration) -> Vec<ToastNotificationSnapshot> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.retain(|_, snapshot| snapshot.expires_at() > now);
        entries.values().cloned().collect()
    }
}
