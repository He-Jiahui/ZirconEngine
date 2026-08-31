use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use super::RandomAlgorithmId;

/// Serializable master-seed authority required to reproduce future stream derivations.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RandomServiceState {
    algorithm: RandomAlgorithmId,
    master_seed: u64,
    master_seed_generation: u64,
}

impl RandomServiceState {
    pub const fn new(
        algorithm: RandomAlgorithmId,
        master_seed: u64,
        master_seed_generation: u64,
    ) -> Self {
        Self {
            algorithm,
            master_seed,
            master_seed_generation,
        }
    }

    pub const fn algorithm(self) -> RandomAlgorithmId {
        self.algorithm
    }

    pub const fn master_seed(self) -> u64 {
        self.master_seed
    }

    pub const fn master_seed_generation(self) -> u64 {
        self.master_seed_generation
    }
}

/// Immutable evidence for a master-seed generation transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RandomSeedReceipt {
    previous_seed: u64,
    seed: u64,
    previous_generation: u64,
    generation: u64,
}

#[derive(Deserialize)]
struct RandomSeedReceiptWire {
    previous_seed: u64,
    seed: u64,
    previous_generation: u64,
    generation: u64,
}

impl<'de> Deserialize<'de> for RandomSeedReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RandomSeedReceiptWire::deserialize(deserializer)?;
        Self::try_new(
            wire.previous_seed,
            wire.seed,
            wire.previous_generation,
            wire.generation,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl RandomSeedReceipt {
    pub const fn try_new(
        previous_seed: u64,
        seed: u64,
        previous_generation: u64,
        generation: u64,
    ) -> Result<Self, RandomSeedReceiptError> {
        match previous_generation.checked_add(1) {
            Some(expected_generation) if expected_generation == generation => {}
            _ => {
                return Err(RandomSeedReceiptError::NonSuccessorGeneration {
                    previous_generation,
                    generation,
                });
            }
        }
        Ok(Self {
            previous_seed,
            seed,
            previous_generation,
            generation,
        })
    }

    pub const fn previous_seed(self) -> u64 {
        self.previous_seed
    }

    pub const fn seed(self) -> u64 {
        self.seed
    }

    pub const fn previous_generation(self) -> u64 {
        self.previous_generation
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomSeedReceiptError {
    #[error(
        "random seed receipt generation {generation} is not the successor of {previous_generation}"
    )]
    NonSuccessorGeneration {
        previous_generation: u64,
        generation: u64,
    },
}
