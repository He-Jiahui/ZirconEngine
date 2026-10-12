//! Versioned deterministic random-stream contracts for simulation-owned state.
//!
//! Seed derivation and generator execution remain in the Runtime random kernel.

#[path = "algorithm.rs"]
mod algorithm;
#[doc(hidden)]
#[path = "assembly.rs"]
pub mod assembly;
#[path = "checkpoint_error.rs"]
mod checkpoint_error;
#[path = "key.rs"]
mod key;
#[path = "service_checkpoint.rs"]
mod service_checkpoint;
#[path = "service_state.rs"]
mod service_state;
#[path = "state.rs"]
mod state;
#[path = "stream_checkpoint.rs"]
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
#[path = "tests/mod.rs"]
mod tests;
