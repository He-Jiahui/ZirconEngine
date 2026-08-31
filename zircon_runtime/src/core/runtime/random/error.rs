use thiserror::Error;

use zr_contracts::random::{RandomSeedReceiptError, RandomServiceCheckpointError, RandomStreamKey};

/// Rejection emitted when the random authority cannot commit a lifecycle operation.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RandomServiceError {
    #[error("random master-seed generation {generation} is exhausted")]
    SeedGenerationExhausted { generation: u64 },
    #[error("random stream {key:?} already has an active mutable lease")]
    StreamAlreadyAcquired { key: RandomStreamKey },
    #[error("random stream registry reached its {capacity}-entry capacity")]
    StreamCapacityExceeded { capacity: usize },
    #[error("random checkpoint is blocked by {active_leases} active stream leases")]
    CheckpointBlocked { active_leases: usize },
    #[error("random reseed is blocked by {active_leases} active stream leases")]
    ReseedBlocked { active_leases: usize },
    #[error("random stream {key:?} cannot be evicted while its lease is active")]
    StreamEvictionBlocked { key: RandomStreamKey },
    #[error("random stream scope eviction is blocked by {active_leases} active leases")]
    StreamScopeEvictionBlocked { active_leases: usize },
    #[error(transparent)]
    CheckpointContract(#[from] RandomServiceCheckpointError),
    #[error(transparent)]
    SeedReceiptContract(#[from] RandomSeedReceiptError),
}
