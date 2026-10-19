use std::{collections::VecDeque, sync::Mutex, time::Duration};

use super::{ToastCenterConfig, ToastNotification, ToastNotificationError};
use crate::core::notifications::NotificationId;

/// Service delivery is FIFO. A pending entry has no deadline until its own native ACK.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastDeliverySnapshot {
    notification: ToastNotification,
    delivery_token: u64,
    expires_at: Option<Duration>,
}
impl ToastDeliverySnapshot {
    pub fn notification(&self) -> &ToastNotification {
        &self.notification
    }
    pub const fn delivery_token(&self) -> u64 {
        self.delivery_token
    }
    pub const fn expires_at(&self) -> Option<Duration> {
        self.expires_at
    }
    pub fn remaining_lifetime_at(&self, now: Duration) -> Duration {
        self.expires_at
            .map(|deadline| deadline.saturating_sub(now))
            .unwrap_or_else(|| self.notification.lifetime())
    }
}

pub struct ToastDeliveryQueue {
    config: ToastCenterConfig,
    state: Mutex<DeliveryState>,
}
struct DeliveryState {
    entries: VecDeque<ToastDeliverySnapshot>,
    next_token: Option<u64>,
    revision: u64,
}
impl ToastDeliveryQueue {
    pub fn new(config: ToastCenterConfig) -> Self {
        Self {
            config,
            state: Mutex::new(DeliveryState {
                entries: VecDeque::new(),
                next_token: Some(1),
                revision: 0,
            }),
        }
    }
    pub fn publish_at(
        &self,
        notification: ToastNotification,
        now: Duration,
    ) -> Result<(), ToastNotificationError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        expire_shown_head(&mut state, now);
        if state
            .entries
            .iter()
            .any(|entry| entry.notification.id() == notification.id())
        {
            return Err(ToastNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if state.entries.len() >= self.config.capacity() {
            return Err(ToastNotificationError::CapacityReached {
                capacity: self.config.capacity(),
            });
        }
        let token = state
            .next_token
            .ok_or(ToastNotificationError::PresentationSequenceExhausted)?;
        state.next_token = token.checked_add(1);
        state.entries.push_back(ToastDeliverySnapshot {
            notification,
            delivery_token: token,
            expires_at: None,
        });
        state.advance_revision();
        Ok(())
    }

    /// Pending/empty queues are stable; acknowledged heads also depend on the live clock.
    pub(crate) fn projection_revision_at(&self, now: Duration) -> (u64, Option<Duration>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        expire_shown_head(&mut state, now);
        let ticking = state
            .entries
            .front()
            .and_then(|head| head.expires_at)
            .map(|_| now);
        (state.revision, ticking)
    }

    pub fn snapshot_at(&self, now: Duration) -> Vec<ToastDeliverySnapshot> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        expire_shown_head(&mut state, now);
        state.entries.iter().cloned().collect()
    }
    /// Only the current pending head can be acknowledged; duplicate/late ACKs cannot restart TTL.
    pub fn acknowledge_at(
        &self,
        id: &NotificationId,
        token: u64,
        now: Duration,
    ) -> Result<bool, ToastNotificationError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        expire_shown_head(&mut state, now);
        let Some(head) = state.entries.front_mut() else {
            return Ok(false);
        };
        if head.notification.id() != id || head.delivery_token != token || head.expires_at.is_some()
        {
            return Ok(false);
        }
        let deadline = now
            .checked_add(head.notification.lifetime())
            .ok_or(ToastNotificationError::PresentationClockOverflow)?;
        head.expires_at = Some(deadline);
        state.advance_revision();
        Ok(true)
    }
    /// Removal is exact-instance bound, including a user close before the first ACK.
    pub fn dismiss(&self, id: &NotificationId, token: u64) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let Some(index) = state
            .entries
            .iter()
            .position(|entry| entry.notification.id() == id && entry.delivery_token == token)
        else {
            return false;
        };
        let removed = state.entries.remove(index).is_some();
        if removed {
            state.advance_revision();
        }
        removed
    }
}
impl DeliveryState {
    fn advance_revision(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("toast projection revision cannot overflow");
    }
}

fn expire_shown_head(state: &mut DeliveryState, now: Duration) {
    while state
        .entries
        .front()
        .and_then(|entry| entry.expires_at)
        .is_some_and(|deadline| deadline <= now)
    {
        state.entries.pop_front();
        state.advance_revision();
    }
}

#[cfg(test)]
#[path = "tests/first_display.rs"]
mod tests;
