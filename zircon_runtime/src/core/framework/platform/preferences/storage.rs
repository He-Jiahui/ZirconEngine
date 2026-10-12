#[path = "storage/snapshot.rs"]
mod snapshot;
#[path = "../storage/storage_contract.rs"]
mod storage_contract;
#[path = "storage/terminal.rs"]
mod terminal;
#[path = "storage/tickets.rs"]
mod tickets;
#[path = "storage/work_deadline.rs"]
mod work_deadline;

pub use snapshot::{PreferenceDurabilityState, PreferenceEviction, PreferenceReadSnapshot};
pub use storage_contract::PreferenceStorage;
pub use terminal::{
    PreferenceMutationCancelError, PreferenceMutationTerminal,
    PreferencePersistenceFailureProjection, PreferenceTicketWaitResult,
};
pub use tickets::{
    PreferenceFlushTicket, PreferenceMutationCancellation, PreferenceMutationSubmission,
    PreferenceMutationTicket,
};
pub use work_deadline::PreferenceWorkDeadline;
