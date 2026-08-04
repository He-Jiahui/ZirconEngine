mod autosave;
mod autosave_adapter;
mod restore_flow;
mod session_guard;

pub use autosave::{
    AUTOSAVE_RETAINED_SNAPSHOT_COUNT, AutosaveDocumentId, AutosaveDocumentState, AutosaveError,
    AutosaveExtension, AutosaveJobPolicy, AutosavePlan, AutosavePolicy, AutosaveScheduler,
    AutosaveStore,
};
pub use autosave_adapter::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentRequest, AutosaveJobAdapter,
    AutosaveSnapshot, AutosaveSnapshotSource, AutosaveWriteResult,
};
pub use restore_flow::{
    RestoreAction, RestoreCandidate, RestoreFlow, RestoreFlowError, RestorePlan, RestoreResolution,
    RestoreStartup,
};
pub use session_guard::{
    SESSION_LOCK_FILE_NAME, SessionGuard, SessionGuardError, SessionLockDurability,
    SessionLockInspection, SessionLockRecord,
};

#[cfg(test)]
mod tests;
