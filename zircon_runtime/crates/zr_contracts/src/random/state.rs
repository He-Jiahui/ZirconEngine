use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use super::{RandomAlgorithmId, RandomSequenceId};

/// Serializable state of a deterministic random stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RandomState {
    algorithm: RandomAlgorithmId,
    state: u64,
    increment: u64,
    draw_index: u64,
}

#[derive(Deserialize)]
struct RandomStateWire {
    algorithm: RandomAlgorithmId,
    state: u64,
    increment: u64,
    draw_index: u64,
}

impl<'de> Deserialize<'de> for RandomState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RandomStateWire::deserialize(deserializer)?;
        Self::new(wire.algorithm, wire.state, wire.increment, wire.draw_index)
            .map_err(serde::de::Error::custom)
    }
}

impl RandomState {
    pub fn new(
        algorithm: RandomAlgorithmId,
        state: u64,
        increment: u64,
        draw_index: u64,
    ) -> Result<Self, RandomStateError> {
        if increment & 1 == 0 {
            return Err(RandomStateError::EvenIncrement);
        }
        Ok(Self::from_valid_parts(
            algorithm, state, increment, draw_index,
        ))
    }

    pub(crate) const fn from_valid_parts(
        algorithm: RandomAlgorithmId,
        state: u64,
        increment: u64,
        draw_index: u64,
    ) -> Self {
        Self {
            algorithm,
            state,
            increment,
            draw_index,
        }
    }

    pub const fn algorithm(self) -> RandomAlgorithmId {
        self.algorithm
    }

    pub const fn generator_state(self) -> u64 {
        self.state
    }

    pub const fn increment(self) -> u64 {
        self.increment
    }

    pub const fn sequence_id(self) -> RandomSequenceId {
        RandomSequenceId::from_uniform_u64(self.increment >> 1)
    }

    pub const fn draw_index(self) -> u64 {
        self.draw_index
    }
}

/// Rejection emitted when a persisted random-stream snapshot is malformed.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomStateError {
    #[error("PCG stream increment must be odd")]
    EvenIncrement,
}
