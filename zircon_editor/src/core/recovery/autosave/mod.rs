mod content_digest;
mod engine_schema;
mod error;
mod identity;
mod journal_range;
mod policy;
mod scheduler;
mod snapshot_provenance;
mod source_digest;
mod store;

pub use content_digest::AutosaveContentDigest;
pub use engine_schema::AutosaveEngineSchema;
pub use error::AutosaveError;
pub use identity::{AutosaveDocumentId, AutosaveExtension};
pub use journal_range::AutosaveJournalRange;
pub use policy::{AutosaveDocumentState, AutosaveJobPolicy, AutosavePlan, AutosavePolicy};
pub use scheduler::AutosaveScheduler;
pub use snapshot_provenance::AutosaveSnapshotProvenance;
pub use source_digest::AutosaveSourceDigest;
pub use store::{AUTOSAVE_RETAINED_SNAPSHOT_COUNT, AutosaveStore};

// The recovery catalog owns metadata semantics but shares the one atomic-publish primitive.
pub(super) use store::write_new_atomically;

#[cfg(test)]
mod tests;
