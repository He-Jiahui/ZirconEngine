use serde::{Deserialize, Serialize};

use super::{RandomState, RandomStreamKey};

/// Persisted progress for one stable random-stream owner.
///
/// The authority generation binds this parked state to the seed era that
/// derived it; service checkpoints reject mixed-generation entries.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RandomStreamCheckpoint {
    key: RandomStreamKey,
    state: RandomState,
    master_seed_generation: u64,
}

impl RandomStreamCheckpoint {
    pub const fn new(
        key: RandomStreamKey,
        state: RandomState,
        master_seed_generation: u64,
    ) -> Self {
        Self {
            key,
            state,
            master_seed_generation,
        }
    }

    pub const fn key(self) -> RandomStreamKey {
        self.key
    }

    pub const fn state(self) -> RandomState {
        self.state
    }

    pub const fn master_seed_generation(self) -> u64 {
        self.master_seed_generation
    }
}
