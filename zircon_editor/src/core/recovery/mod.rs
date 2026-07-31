mod autosave;

pub use autosave::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveError, AutosaveExtension, AutosaveJobPolicy,
    AutosavePlan, AutosavePolicy, AutosaveScheduler, AutosaveStore,
    AUTOSAVE_RETAINED_SNAPSHOT_COUNT,
};

#[cfg(test)]
mod tests;
