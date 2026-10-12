#[path = "backend_kind.rs"]
mod backend_kind;
#[path = "error.rs"]
mod error;
#[path = "key.rs"]
mod key;
#[path = "storage.rs"]
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
