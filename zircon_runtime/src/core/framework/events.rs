//! Event DTOs shared by framework contracts and runtime delivery.

use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineEvent {
    pub topic: String,
    pub payload: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineEventDeliveryPolicy {
    Lossless,
    BoundedDropOldest { capacity: NonZeroUsize },
    Latest,
}

pub const DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL: NonZeroU64 =
    NonZeroU64::new(64).expect("event timing sample interval must be non-zero");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventBusDiagnosticsMode {
    Enabled,
    Sampled { every: NonZeroU64 },
    Disabled,
}

impl Default for EventBusDiagnosticsMode {
    fn default() -> Self {
        Self::Sampled {
            every: DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL,
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EngineEventReceiveError {
    #[error("event subscription is disconnected")]
    Disconnected,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EngineEventTryReceiveError {
    #[error("event subscription is empty")]
    Empty,
    #[error("event subscription is disconnected")]
    Disconnected,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EngineEventReceiveTimeoutError {
    #[error("event subscription receive timed out")]
    Timeout,
    #[error("event subscription is disconnected")]
    Disconnected,
}

pub trait EngineEventSubscription: Send + Sync {
    fn recv(&self) -> Result<Arc<EngineEvent>, EngineEventReceiveError>;
    fn try_recv(&self) -> Result<Arc<EngineEvent>, EngineEventTryReceiveError>;
    fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Arc<EngineEvent>, EngineEventReceiveTimeoutError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EventBusDiagnosticsSnapshot {
    pub enabled: bool,
    pub routine_timing_sample_interval: u64,
    pub topics: u64,
    pub subscribers: u64,
    pub published: u64,
    pub delivered: u64,
    pub dropped: u64,
    pub disconnected: u64,
    pub queued: u64,
    pub peak_queued: u64,
    pub waiting_receivers: u64,
    pub waiting_publishers: u64,
    pub queue_age_samples: u64,
    pub total_queue_age_ms: f64,
    pub max_queue_age_ms: f64,
    pub publish_samples: u64,
    pub total_publish_ms: f64,
    pub max_publish_ms: f64,
    pub delivery_lock_wait_samples: u64,
    pub total_delivery_lock_wait_ms: f64,
    pub max_delivery_lock_wait_ms: f64,
}
