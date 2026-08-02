//! Neutral runtime platform contracts shared across assembly and host domains.

mod module_identity;
mod preferences;
mod runtime_target_mode;

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
