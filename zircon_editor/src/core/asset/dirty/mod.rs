//! Document dirty-state projection backed by Editor03 saved-top authority.

mod error;
mod external_effect_id;
mod registry;
mod save_batch;
mod save_job_adapter;

pub use error::DirtyRegistryError;
pub use external_effect_id::{DirtyExternalEffectId, DirtyExternalEffectIdError};
pub use registry::{
    DirtyDocumentSnapshot, DirtyExternalEffectRevision, DirtyRegistry, DirtyRegistryCursor,
    DirtyRegistryDelta,
};
pub use save_batch::{
    SaveDirtyViewCandidate, SaveDirtyViewCompletion, SaveDirtyViewFailure,
    SaveDirtyViewFailureKind, SaveDirtyViewIntent, SaveDirtyViewOutcome,
    SaveDirtyViewOutcomeStatus, SaveDirtyViewsApplyError, SaveDirtyViewsPreflightError,
    SaveDirtyViewsPreflightErrorKind, SaveDirtyViewsPreflightReport, SaveDirtyViewsRequest,
    SaveDirtyViewsResult,
};
pub use save_job_adapter::{
    SaveDirtyViewExecutor, SaveDirtyViewsAdmissionError, SaveDirtyViewsCompletionBatch,
    SaveDirtyViewsCompletionPoll, SaveDirtyViewsJobAdapter,
    DEFAULT_SAVE_DIRTY_VIEWS_COMPLETION_BUDGET,
};

#[cfg(test)]
mod tests;
