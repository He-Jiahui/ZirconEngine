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
}
