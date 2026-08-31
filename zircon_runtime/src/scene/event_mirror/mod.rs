mod error;
mod registration;
mod subscription;

pub use error::RuntimeEventMirrorError;
pub use registration::{RuntimeEventMirrorDescriptor, RuntimeEventMirrorRegistration};
pub(crate) use registration::{
    RuntimeEventMirrorLifecycleDiagnostics, RuntimeEventMirrorReclaimReport,
};
pub use subscription::RuntimeEventMirrorSubscription;
pub(crate) use subscription::{
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS, RuntimeEventMirrorSubscriptionHandle,
    RuntimeEventMirrorSubscriptionRecord,
};
pub(crate) use subscription::{RuntimeEventMirrorDrainPage, RuntimeEventMirrorPayload};

pub(crate) use registration::RuntimeEventMirrorRegistry;
