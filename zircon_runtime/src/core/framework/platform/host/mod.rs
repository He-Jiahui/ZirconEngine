mod backend;
mod backend_kind;
mod backend_request_error;
mod descriptor;
mod evidence;
mod evidence_error;
mod failure_reason;
mod generation;
mod health;
mod instance_id;
mod lifecycle_state;
mod observed_capabilities;
mod operation_id;
mod quiesce_request;
mod snapshot;
mod terminal_result;
mod thread_affinity;

pub use backend::PlatformHostBackend;
pub use backend_kind::PlatformHostBackendKind;
pub use backend_request_error::PlatformHostBackendRequestError;
pub use descriptor::PlatformHostDescriptor;
pub use evidence::{PlatformHostEvidence, PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES};
pub use evidence_error::PlatformHostEvidenceError;
pub use failure_reason::PlatformHostFailureReason;
pub use generation::PlatformHostGeneration;
pub use health::PlatformHostHealth;
pub use instance_id::PlatformHostInstanceId;
pub use lifecycle_state::PlatformHostLifecycleState;
pub use observed_capabilities::PlatformHostObservedCapabilities;
pub use operation_id::PlatformHostOperationId;
pub use quiesce_request::PlatformHostQuiesceRequest;
pub use snapshot::PlatformHostSnapshot;
pub use terminal_result::PlatformHostTerminalResult;
pub use thread_affinity::PlatformHostThreadAffinity;

#[cfg(test)]
mod tests;
