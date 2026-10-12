#[path = "backend.rs"]
mod backend;
#[path = "backend_kind.rs"]
mod backend_kind;
#[path = "backend_request_error.rs"]
mod backend_request_error;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "evidence.rs"]
mod evidence;
#[path = "evidence_error.rs"]
mod evidence_error;
#[path = "failure_reason.rs"]
mod failure_reason;
#[path = "generation.rs"]
mod generation;
#[path = "health.rs"]
mod health;
#[path = "instance_id.rs"]
mod instance_id;
#[path = "lifecycle_state.rs"]
mod lifecycle_state;
#[path = "observed_capabilities.rs"]
mod observed_capabilities;
#[path = "operation_id.rs"]
mod operation_id;
#[path = "quiesce_request.rs"]
mod quiesce_request;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "terminal_result.rs"]
mod terminal_result;
#[path = "thread_affinity.rs"]
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
#[path = "tests/cases.rs"]
mod tests;
