//! 为显式注册的可序列化事件提供独立的有界镜像队列，供外部订阅分页排空；普通 ECS 事件队列按自身代际发布。
mod error;
mod registration;
mod subscription;

pub use error::RuntimeEventMirrorError;
pub use registration::{RuntimeEventMirrorDescriptor, RuntimeEventMirrorRegistration};
pub(crate) use registration::{
    RuntimeEventMirrorLifecycleDiagnostics, RuntimeEventMirrorReclaimReport,
};
pub use subscription::RuntimeEventMirrorSubscription;
pub(crate) use subscription::{RuntimeEventMirrorDrainPage, RuntimeEventMirrorPayload};
pub(crate) use subscription::{
    RuntimeEventMirrorSubscriptionHandle, RuntimeEventMirrorSubscriptionRecord,
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
};

pub(crate) use registration::RuntimeEventMirrorRegistry;
