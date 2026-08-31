//! Versioned deterministic random-stream contracts for simulation-owned state.
//!
//! Seed derivation and generator execution remain in the Runtime random kernel.

mod algorithm;
#[doc(hidden)]
pub mod assembly;
mod checkpoint_error;
mod key;
mod service_checkpoint;
mod service_state;
mod state;
mod stream_checkpoint;

pub use algorithm::{
    RandomAlgorithmId, RandomAlgorithmIdError, RandomSequenceId, RandomSequenceIdError,
};
pub use checkpoint_error::RandomServiceCheckpointError;
pub use key::{
    RandomEntityKey, RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey,
};
pub use service_checkpoint::RandomServiceCheckpoint;
pub use service_state::{RandomSeedReceipt, RandomSeedReceiptError, RandomServiceState};
pub use state::{RandomState, RandomStateError};
pub use stream_checkpoint::RandomStreamCheckpoint;

#[cfg(test)]
mod tests;
