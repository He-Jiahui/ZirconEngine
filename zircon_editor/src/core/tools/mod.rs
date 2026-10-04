mod authority_state;
mod claim;
mod identity;
mod input_capture;
mod limits;
mod resource_catalog;
mod resource_set;
mod scheduler;
mod snapshot;
mod transition;

pub use authority_state::{ToolAuthorityState, ToolShutdownOutcome};
pub use claim::{
    AcquireDenial, AcquireOutcome, ReleaseOutcome, ToolLeaseHandle, ToolLifecycleEvent,
    ToolOwnerRevokeOutcome, ToolRequestHandle, ToolScheduleReport, WithdrawOutcome,
};
pub use identity::{
    ToolDefinitionId, ToolDefinitionIdError, ToolInstanceId, ToolInstanceIdError, ToolLeaseId,
    ToolOwnerGeneration, ToolRequestId, MAX_TOOL_DEFINITION_ID_BYTES, MAX_TOOL_INSTANCE_ID_BYTES,
};
pub use input_capture::{
    ToolInputCaptureDenial, ToolInputCaptureDisposition, ToolInputCaptureEndOutcome,
    ToolInputCaptureEvent, ToolInputCaptureHandle, ToolInputCaptureId, ToolInputCaptureOutcome,
    ToolInputCaptureOwner, ToolInputCapturePriority, ToolInputCaptureRequest, ToolInputScope,
    ToolInputSource, DEFAULT_MAX_ACTIVE_TOOL_INPUT_CAPTURES,
};
pub use limits::{ToolQueueLimits, DEFAULT_MAX_SET_QUEUE, DEFAULT_MAX_SINGLE_QUEUE_PER_RESOURCE};
pub(crate) use resource_catalog::ToolResourceCatalog;
pub use resource_catalog::{
    ToolResourceCatalogError, ToolResourceChannelPolicy, ToolResourceKindDeclaration,
    ToolResourceKindRegistration, ToolResourceKindRegistrationError,
    DEFAULT_MAX_REGISTERED_TOOL_RESOURCE_KINDS, DEFAULT_MAX_TOOL_RESOURCE_KINDS_PER_OWNER,
};
pub use resource_set::{
    ToolResourceChannelId, ToolResourceIdError, ToolResourceKey, ToolResourceKeyError,
    ToolResourceKindId, ToolResourceSet, ToolResourceSetError, ToolScope, ToolScopeKind,
    MAX_TOOL_RESOURCE_IDENTIFIER_BYTES,
};
pub(crate) use scheduler::ToolScheduler;
pub use snapshot::{ToolResourceStateSnapshot, ToolSchedulerStateSnapshot};
pub use transition::{ToolTransitionBatch, ToolTransitionRevision};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
