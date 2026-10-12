#[path = "batch.rs"]
mod batch;
#[path = "cursor.rs"]
mod cursor;
#[path = "identity.rs"]
mod identity;
#[path = "journal.rs"]
mod journal;
#[path = "observation.rs"]
mod observation;
#[path = "source.rs"]
mod source;

pub use batch::TaskDiagnosticBatch;
pub use cursor::TaskDiagnosticCursor;
pub use identity::TaskDiagnosticIdentity;
pub(super) use journal::TaskDiagnosticJournal;
pub use observation::{
    TaskDiagnosticKind, TaskDiagnosticObservation, TaskDiagnosticSeverity,
    MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES,
};
pub use source::TaskDiagnosticSource;

pub const TASK_DIAGNOSTIC_RETENTION_CAPACITY: usize = 256;
pub const TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES: usize = 64;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
