use thiserror::Error;

use super::RandomAlgorithmId;

/// Rejection emitted when a persisted random-service checkpoint is not canonical.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomServiceCheckpointError {
    #[error("unsupported random-service checkpoint format version {version}")]
    UnsupportedFormatVersion { version: u16 },
    #[error("random-service checkpoint contains at least {actual} streams and exceeds the maximum of {max} streams")]
    TooManyStreams { max: usize, actual: usize },
    #[error("random stream checkpoint keys are not strictly increasing at index {index}")]
    NonCanonicalStreamOrder { index: usize },
    #[error(
        "random stream checkpoint at index {index} uses authority generation {stream_generation}, expected {service_generation}"
    )]
    StreamAuthorityGenerationMismatch {
        index: usize,
        service_generation: u64,
        stream_generation: u64,
    },
    #[error(
        "random stream checkpoint at index {index} uses {stream_algorithm:?}, expected {service_algorithm:?}"
    )]
    StreamAlgorithmMismatch {
        index: usize,
        service_algorithm: RandomAlgorithmId,
        stream_algorithm: RandomAlgorithmId,
    },
}
