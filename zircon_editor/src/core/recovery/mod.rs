mod autosave;
mod autosave_adapter;
mod autosave_catalog;
mod autosave_service;
mod restore_flow;
mod session_guard;

pub use autosave::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveError, AutosaveExtension, AutosaveJobPolicy,
    AutosavePlan, AutosavePolicy, AutosaveScheduler, AutosaveStore,
    AUTOSAVE_RETAINED_SNAPSHOT_COUNT,
};
pub use autosave_adapter::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentRequest, AutosaveJobAdapter,
    AutosaveSnapshot, AutosaveSnapshotSource, AutosaveWriteResult,
    DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};
pub use autosave_catalog::AutosaveSourcePath;
pub(crate) use autosave_service::{EditorAutosavePoll, EditorAutosaveService};
pub use restore_flow::{
    RestoreAction, RestoreCandidate, RestoreFlow, RestoreFlowError, RestorePlan, RestoreResolution,
    RestoreStartup,
};
pub use session_guard::{
    SessionGuard, SessionGuardAdmission, SessionGuardError, SessionGuardResidual,
    SessionLockDurability, SessionLockInspection, SessionLockRecord, SESSION_LOCK_FILE_NAME,
};

#[cfg(test)]
mod tests;
