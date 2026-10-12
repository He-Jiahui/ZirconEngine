#[path = "content_digest.rs"]
mod content_digest;
#[path = "engine_schema.rs"]
mod engine_schema;
#[path = "error.rs"]
mod error;
#[path = "identity.rs"]
mod identity;
#[path = "journal_range.rs"]
mod journal_range;
#[path = "policy.rs"]
mod policy;
#[path = "scheduler.rs"]
mod scheduler;
#[path = "snapshot_provenance.rs"]
mod snapshot_provenance;
#[path = "source_digest.rs"]
mod source_digest;
#[path = "store.rs"]
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
pub use store::{AutosaveStore, AUTOSAVE_RETAINED_SNAPSHOT_COUNT};

// The recovery catalog owns metadata semantics but shares the one atomic-publish primitive.
pub(super) use store::write_new_atomically;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
