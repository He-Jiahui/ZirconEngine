mod adapter;
mod model;
mod write_job;

pub use adapter::AutosaveJobAdapter;
pub use model::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentOutcome,
    AutosaveDocumentOutcomeKind, AutosaveDocumentRequest, AutosaveFailureStage,
    AutosaveHealthTelemetry, AutosaveRetryability, AutosaveSnapshot, AutosaveSnapshotSource,
    AutosaveWriteResult, DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};
