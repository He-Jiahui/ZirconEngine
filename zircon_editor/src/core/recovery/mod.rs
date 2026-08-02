mod autosave;
mod autosave_adapter;

pub use autosave::{
    AUTOSAVE_RETAINED_SNAPSHOT_COUNT, AutosaveDocumentId, AutosaveDocumentState, AutosaveError,
    AutosaveExtension, AutosaveJobPolicy, AutosavePlan, AutosavePolicy, AutosaveScheduler,
    AutosaveStore,
};
pub use autosave_adapter::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentRequest, AutosaveJobAdapter,
    AutosaveSnapshot, AutosaveSnapshotSource, AutosaveWriteResult,
};

#[cfg(test)]
mod tests;
