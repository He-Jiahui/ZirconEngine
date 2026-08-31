use thiserror::Error;

use super::RandomAlgorithmId;

/// Rejection emitted when a persisted random-service checkpoint is not canonical.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomServiceCheckpointError {
    #[error("unsupported random-service checkpoint format version {version}")]
    UnsupportedFormatVersion { version: u16 },
    #[error("random stream checkpoint keys are not strictly increasing at index {index}")]
    NonCanonicalStreamOrder { index: usize },
    #[error(
        "random stream checkpoint at index {index} uses {stream_algorithm:?}, expected {service_algorithm:?}"
    )]
    StreamAlgorithmMismatch {
        index: usize,
        service_algorithm: RandomAlgorithmId,
        stream_algorithm: RandomAlgorithmId,
    },
}
