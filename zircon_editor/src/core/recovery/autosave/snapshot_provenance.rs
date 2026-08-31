use serde::{Deserialize, Serialize};

use super::{AutosaveEngineSchema, AutosaveJournalRange, AutosaveSourceDigest};

/// Immutable capture facts attached to one autosave snapshot commit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveSnapshotProvenance {
    base_generation: u64,
    source_digest: AutosaveSourceDigest,
    journal_range: AutosaveJournalRange,
    engine_schema: AutosaveEngineSchema,
}

impl AutosaveSnapshotProvenance {
    /// `base_generation` is the document dirty-registry generation observed by
    /// the capture owner. It is deliberately distinct from a journal offset.
    pub fn capture(base_generation: u64, source_digest: AutosaveSourceDigest) -> Self {
        Self {
            base_generation,
            source_digest,
            journal_range: AutosaveJournalRange::Unavailable,
            engine_schema: AutosaveEngineSchema::current(),
        }
    }

    pub const fn base_generation(&self) -> u64 {
        self.base_generation
    }

    pub fn source_digest(&self) -> &AutosaveSourceDigest {
        &self.source_digest
    }

    pub fn journal_range(&self) -> &AutosaveJournalRange {
        &self.journal_range
    }

    pub fn engine_schema(&self) -> &AutosaveEngineSchema {
        &self.engine_schema
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.source_digest.is_valid() && self.engine_schema.is_current()
    }
}
