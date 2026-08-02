mod backend_kind;
mod error;
mod key;
mod storage;

pub use backend_kind::PreferenceStorageBackendKind;
pub use error::{PreferenceStorageError, PreferenceStorageErrorKind, PreferenceStorageOperation};
pub use key::{PreferenceKey, PreferenceKeyError, PreferenceKeyErrorKind};
pub use storage::{
    PreferenceDurabilityState, PreferenceEviction, PreferenceFlushTicket,
    PreferenceMutationCancelError, PreferenceMutationCancellation, PreferenceMutationSubmission,
    PreferenceMutationTerminal, PreferenceMutationTicket, PreferencePersistenceFailureProjection,
    PreferenceReadSnapshot, PreferenceStorage, PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
