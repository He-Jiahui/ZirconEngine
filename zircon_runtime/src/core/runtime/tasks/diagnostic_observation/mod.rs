mod batch;
mod cursor;
mod identity;
mod journal;
mod observation;
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
mod tests;
