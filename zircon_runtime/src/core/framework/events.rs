//! Event DTOs shared by framework contracts and runtime delivery.

use std::fmt;
use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// 由 CoreHandle 发布的主题消息；总线按 topic 分发，同一订阅者的队列策略决定积压处理。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineEvent {
    pub topic: String,
    pub payload: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Every policy is bounded by outstanding delivery count and frozen buffer bytes.
/// Reliable rejects pressure; lossy modes may replace queued deliveries only.
pub enum EngineEventDeliveryPolicy {
    Reliable { limits: EventRetentionLimits },
    DropOldest { limits: EventRetentionLimits },
    Latest { max_retained_bytes: NonZeroUsize },
}

pub const DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL: NonZeroU64 =
    NonZeroU64::new(64).expect("event timing sample interval must be non-zero");

/// 只影响总线统计采样，不改变事件投递；Disabled 下诊断快照不会记录流量计数。
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

/// EventBus::subscribe 返回的订阅句柄。调用方必须持有它；Drop 会注销并清空待收队列。
/// recv/recv_timeout 适合等待事件，try_recv 适合宿主循环按自己的节奏清空队列。
pub trait EngineEventSubscription: Send + Sync {
    fn recv(&self) -> Result<EngineEventDelivery, EngineEventReceiveError>;
    fn try_recv(&self) -> Result<EngineEventDelivery, EngineEventTryReceiveError>;
    fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<EngineEventDelivery, EngineEventReceiveTimeoutError>;
}

/// EventBus::diagnostic_report 的瞬时观测值；时间字段是采样累计值，需与对应 samples 配对解读。
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

/// Count includes queued deliveries and received handles until their final clone drops.
/// Bytes are exact owned frozen topic and encoded payload buffers; allocator metadata
/// for admitted queue storage is separately bounded by count and registry limits.
/// Caller-owned input, decoded values, clone handles, and in-flight preparation
/// are outside the accepted frozen-buffer ledger; this is not an RSS ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventRetentionLimits {
    pub max_events: NonZeroUsize,
    pub max_bytes: NonZeroUsize,
}

pub const DEFAULT_EVENT_RETENTION_EVENTS: usize = 1024;
pub const DEFAULT_EVENT_RETENTION_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_EVENT_BUS_GLOBAL_EVENTS: usize = 16 * 1024;
pub const DEFAULT_EVENT_BUS_GLOBAL_BYTES: usize = 64 * 1024 * 1024;
pub const DEFAULT_EVENT_BUS_TOPIC_EVENTS: usize = 8 * 1024;
pub const DEFAULT_EVENT_BUS_TOPIC_BYTES: usize = 32 * 1024 * 1024;
pub const DEFAULT_EVENT_BUS_MAX_TOPICS: usize = 1024;
pub const DEFAULT_EVENT_BUS_MAX_SUBSCRIBERS: usize = 1024;
pub const DEFAULT_EVENT_TOPIC_NAME_BYTES: usize = 1024;
pub const DEFAULT_EVENT_PAYLOAD_BYTES: usize = 1024 * 1024;

impl Default for EventRetentionLimits {
    fn default() -> Self {
        Self {
            max_events: NonZeroUsize::new(DEFAULT_EVENT_RETENTION_EVENTS).unwrap(),
            max_bytes: NonZeroUsize::new(DEFAULT_EVENT_RETENTION_BYTES).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventBusLimits {
    pub global: EventRetentionLimits,
    pub topic: EventRetentionLimits,
    pub max_topics: NonZeroUsize,
    pub max_subscribers: NonZeroUsize,
    pub max_topic_name_bytes: NonZeroUsize,
    pub max_event_payload_bytes: NonZeroUsize,
}

impl Default for EventBusLimits {
    fn default() -> Self {
        Self {
            global: EventRetentionLimits {
                max_events: NonZeroUsize::new(DEFAULT_EVENT_BUS_GLOBAL_EVENTS).unwrap(),
                max_bytes: NonZeroUsize::new(DEFAULT_EVENT_BUS_GLOBAL_BYTES).unwrap(),
            },
            topic: EventRetentionLimits {
                max_events: NonZeroUsize::new(DEFAULT_EVENT_BUS_TOPIC_EVENTS).unwrap(),
                max_bytes: NonZeroUsize::new(DEFAULT_EVENT_BUS_TOPIC_BYTES).unwrap(),
            },
            max_topics: NonZeroUsize::new(DEFAULT_EVENT_BUS_MAX_TOPICS).unwrap(),
            max_subscribers: NonZeroUsize::new(DEFAULT_EVENT_BUS_MAX_SUBSCRIBERS).unwrap(),
            max_topic_name_bytes: NonZeroUsize::new(DEFAULT_EVENT_TOPIC_NAME_BYTES).unwrap(),
            max_event_payload_bytes: NonZeroUsize::new(DEFAULT_EVENT_PAYLOAD_BYTES).unwrap(),
        }
    }
}

impl EngineEventDeliveryPolicy {
    pub(crate) fn limits(self) -> EventRetentionLimits {
        match self {
            Self::Reliable { limits } | Self::DropOldest { limits } => limits,
            Self::Latest { max_retained_bytes } => EventRetentionLimits {
                max_events: NonZeroUsize::new(1).unwrap(),
                max_bytes: max_retained_bytes,
            },
        }
    }
    pub(crate) fn allows_replacement(self) -> bool {
        !matches!(self, Self::Reliable { .. })
    }
}

/// Neutral, immutable serialized envelope. JSON decoding allocates caller-owned data.
#[derive(Debug)]
pub struct FrozenEngineEvent {
    pub(crate) topic: Box<str>,
    pub(crate) payload: Box<[u8]>,
}

pub(crate) trait EngineEventRetentionLease: Send + Sync {
    /// Called exactly once after retention admission is committed.
    fn activate_charge(&self);
}

#[derive(Clone)]
pub struct EngineEventDelivery {
    pub(crate) event: Arc<FrozenEngineEvent>,
    pub(crate) lease: Arc<dyn EngineEventRetentionLease>,
}

impl EngineEventDelivery {
    pub fn topic(&self) -> &str {
        &self.event.topic
    }
    pub fn payload_bytes(&self) -> &[u8] {
        &self.event.payload
    }
    pub fn retained_bytes(&self) -> usize {
        self.event.topic.len() + self.event.payload.len()
    }
    pub fn decode_payload(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_slice(self.payload_bytes())
    }
    pub fn shares_payload_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.event, &other.event)
    }
}

impl fmt::Debug for EngineEventDelivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineEventDelivery")
            .field("topic", &self.topic())
            .field("retained_bytes", &self.retained_bytes())
            .finish()
    }
}
impl PartialEq for EngineEventDelivery {
    fn eq(&self, other: &Self) -> bool {
        self.topic() == other.topic() && self.payload_bytes() == other.payload_bytes()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventBudgetScope {
    Subscriber,
    Topic,
    Global,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineEventPublishRejection {
    NoSubscribers,
    Closed,
    Poisoned,
    InvalidTopic,
    PayloadTooLarge,
    SerializationFailed,
    Backpressured { scope: EventBudgetScope },
}

#[derive(Debug, PartialEq)]
#[must_use]
pub struct EngineEventPublishRejected {
    pub reason: EngineEventPublishRejection,
    pub event: EngineEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub struct EngineEventPublishReceipt {
    pub subscribers: usize,
    pub replaced_events: usize,
    pub replaced_bytes: usize,
    pub admitted_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineEventSubscribeError {
    Closed,
    Poisoned,
    InvalidTopic,
    RegistryFull,
    IdentifierExhausted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventBusRetentionSnapshot {
    pub closed: bool,
    pub poisoned: bool,
    pub retained_events: usize,
    pub retained_bytes: usize,
    pub peak_retained_events: usize,
    pub peak_retained_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[must_use]
pub struct EventBusCloseReceipt {
    pub drained_events: usize,
    pub retention: EventBusRetentionSnapshot,
}
