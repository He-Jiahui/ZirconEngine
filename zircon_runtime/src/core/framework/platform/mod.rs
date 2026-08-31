//! Neutral runtime platform contracts shared across assembly and host domains.

mod application_lifecycle;
mod event_loop_scheduler;
mod host;
mod module_identity;
mod preferences;
mod runtime_target_mode;

pub use application_lifecycle::{
    ApplicationActivationState, ApplicationLifecycleGeneration, ApplicationLifecycleOperation,
    ApplicationLifecycleOperationId, ApplicationLifecycleSnapshot, ApplicationLifecycleState,
    ApplicationLifecycleTerminalResult, ApplicationSurfaceAvailability,
};
pub use event_loop_scheduler::{
    EventLoopBackgroundPolicy, EventLoopClockDomain, EventLoopControlFlow, EventLoopHostWakeReason,
    EventLoopWakeRequest, EventLoopWakeSource,
};
pub use host::{
    PlatformHostBackend, PlatformHostBackendKind, PlatformHostBackendRequestError,
    PlatformHostDescriptor, PlatformHostEvidence, PlatformHostEvidenceError,
    PlatformHostFailureReason, PlatformHostGeneration, PlatformHostHealth, PlatformHostInstanceId,
    PlatformHostLifecycleState, PlatformHostObservedCapabilities, PlatformHostOperationId,
    PlatformHostQuiesceRequest, PlatformHostSnapshot, PlatformHostTerminalResult,
    PlatformHostThreadAffinity, PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES,
};
pub use module_identity::PLATFORM_MODULE_NAME;
pub use preferences::{
    PreferenceDurabilityState, PreferenceEviction, PreferenceFlushTicket, PreferenceKey,
    PreferenceKeyError, PreferenceKeyErrorKind, PreferenceMutationCancelError,
    PreferenceMutationCancellation, PreferenceMutationSubmission, PreferenceMutationTerminal,
    PreferenceMutationTicket, PreferencePersistenceFailureProjection, PreferenceReadSnapshot,
    PreferenceStorage, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation, PreferenceTicketWaitResult,
    PreferenceWorkDeadline,
};
pub use runtime_target_mode::RuntimeTargetMode;
