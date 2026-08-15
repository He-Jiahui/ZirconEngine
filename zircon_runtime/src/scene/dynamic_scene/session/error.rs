use thiserror::Error;
use zircon_runtime_interface::serialization::CanonicalTextWriteError;

use super::super::DynamicSceneError;

#[derive(Debug, Error)]
pub enum RuntimeSessionArchiveError {
    #[error("runtime session archive I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("runtime session archive parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    CanonicalText(#[from] CanonicalTextWriteError),
    #[error(transparent)]
    DynamicScene(#[from] DynamicSceneError),
    #[error("unsupported runtime session archive format version {actual}; expected {expected}")]
    UnsupportedFormatVersion { expected: u32, actual: u32 },
    #[error("runtime session archive contains duplicate slot `{slot_id}`")]
    DuplicateSlotId { slot_id: String },
    #[error("runtime session archive has no slot `{slot_id}`")]
    MissingSlot { slot_id: String },
    #[error("runtime session slot id cannot be empty")]
    EmptySlotId,
    #[error("runtime session slot id `{slot_id}` is not canonical; use `{canonical}`")]
    NonCanonicalSlotId { slot_id: String, canonical: String },
    #[error(
        "runtime session archive revision {artifact_revision} is older than committed lineage revision {committed_revision}"
    )]
    StaleArtifactRevision {
        artifact_revision: u64,
        committed_revision: u64,
    },
    #[error(
        "runtime session archive merge plan is stale (expected generation {expected_generation} revision {expected_revision}, found generation {current_generation} revision {current_revision})"
    )]
    StaleMergePlan {
        expected_generation: u64,
        expected_revision: u64,
        current_generation: u64,
        current_revision: u64,
    },
    #[error(
        "runtime session archive prune plan is stale (expected generation {expected_generation} revision {expected_revision}, found generation {current_generation} revision {current_revision})"
    )]
    StalePrunePlan {
        expected_generation: u64,
        expected_revision: u64,
        current_generation: u64,
        current_revision: u64,
    },
    #[error(
        "runtime session archive capture-retention plan is stale (expected generation {expected_generation} revision {expected_revision}, found generation {current_generation} revision {current_revision})"
    )]
    StaleCaptureRetentionPlan {
        expected_generation: u64,
        expected_revision: u64,
        current_generation: u64,
        current_revision: u64,
    },
    #[error(
        "runtime session archive path changed after save preparation (expected commit {expected_commit}, found {committed_commit})"
    )]
    StalePathCommit {
        expected_commit: u64,
        committed_commit: u64,
    },
    #[error(
        "runtime session archive path write intent {write_generation} was superseded by {current_generation}"
    )]
    StalePathWrite {
        write_generation: u64,
        current_generation: u64,
    },
    #[error(
        "runtime session archive artifact exceeds the {limit_bytes}-byte limit (found at least {estimated_bytes} bytes)"
    )]
    ArtifactTooLarge {
        estimated_bytes: usize,
        limit_bytes: usize,
    },
}
