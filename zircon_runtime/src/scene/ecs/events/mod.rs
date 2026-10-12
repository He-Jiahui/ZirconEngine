//! 场景事件的类型注册、读游标和生命周期 lease 入口。
//!

#[path = "cursor.rs"]
mod cursor;
#[path = "id.rs"]
mod id;
#[path = "lease.rs"]
mod lease;
#[path = "metrics.rs"]
mod metrics;
#[path = "observer.rs"]
mod observer;
#[path = "queue.rs"]
mod queue;
#[path = "store.rs"]
mod store;
#[path = "subscription.rs"]
mod subscription;

pub use cursor::{EventCursor, EventReadIter};
pub use id::{Event, EventTypeId};
pub use lease::EventReaderLease;
pub use metrics::{
    EventCapacityMetrics, EventPayloadProfile, EventPayloadStorage,
    EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
pub(crate) use observer::{EventObserverHandle, EventObserverId};
pub use queue::Events;
pub use store::EventStore;
pub(in crate::scene) use store::EventWriterGrant;
pub use subscription::{EventSubscription, EventSubscriptionStatus};
