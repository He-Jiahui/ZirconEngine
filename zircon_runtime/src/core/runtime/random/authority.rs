use std::sync::{Mutex, MutexGuard};

use zr_contracts::random::RandomSeedReceipt;
use zr_contracts::random::{RandomAlgorithmId, RandomServiceState, RandomStreamCheckpoint};

use super::registry::RandomStreamRegistry;
use super::{RandomServiceError, RandomServiceLimits};

/// Complete seed, generation and stream-state authority retained by active leases.
#[derive(Debug)]
pub(crate) struct RandomAuthority {
    algorithm: RandomAlgorithmId,
    seed: Mutex<RandomSeedAuthority>,
    registry: RandomStreamRegistry,
}

#[derive(Debug)]
struct RandomSeedAuthority {
    master_seed: u64,
    generation: u64,
}

impl RandomAuthority {
    pub(crate) fn new(master_seed: u64, limits: RandomServiceLimits) -> Self {
        Self {
            algorithm: RandomAlgorithmId::Pcg32XshRrV1,
            seed: Mutex::new(RandomSeedAuthority {
                master_seed,
                generation: 0,
            }),
            registry: RandomStreamRegistry::new(limits),
        }
    }

    pub(crate) fn from_state(state: RandomServiceState, limits: RandomServiceLimits) -> Self {
        Self {
            algorithm: state.algorithm(),
            seed: Mutex::new(RandomSeedAuthority {
                master_seed: state.master_seed(),
                generation: state.master_seed_generation(),
            }),
            registry: RandomStreamRegistry::new(limits),
        }
    }

    pub(crate) fn from_checkpoint(
        state: RandomServiceState,
        limits: RandomServiceLimits,
        streams: Vec<RandomStreamCheckpoint>,
    ) -> Result<Self, RandomServiceError> {
        Ok(Self {
            algorithm: state.algorithm(),
            seed: Mutex::new(RandomSeedAuthority {
                master_seed: state.master_seed(),
                generation: state.master_seed_generation(),
            }),
            registry: RandomStreamRegistry::from_checkpoints(limits, streams)?,
        })
    }

    pub(crate) const fn algorithm(&self) -> RandomAlgorithmId {
        self.algorithm
    }

    pub(crate) fn master_seed(&self) -> u64 {
        self.lock_seed().master_seed
    }

    pub(crate) fn master_seed_generation(&self) -> u64 {
        self.lock_seed().generation
    }

    pub(crate) fn snapshot(&self) -> RandomServiceState {
        let seed = self.lock_seed();
        RandomServiceState::new(self.algorithm, seed.master_seed, seed.generation)
    }

    pub(crate) fn registry(&self) -> &RandomStreamRegistry {
        &self.registry
    }

    pub(crate) fn reseed(&self, master_seed: u64) -> Result<RandomSeedReceipt, RandomServiceError> {
        self.reseed_with_observer(master_seed, || {})
    }

    #[cfg(test)]
    pub(crate) fn reseed_with_test_observer(
        &self,
        master_seed: u64,
        on_enter: impl FnOnce(),
    ) -> Result<RandomSeedReceipt, RandomServiceError> {
        self.reseed_with_observer(master_seed, on_enter)
    }

    fn reseed_with_observer(
        &self,
        master_seed: u64,
        on_enter: impl FnOnce(),
    ) -> Result<RandomSeedReceipt, RandomServiceError> {
        match self.registry.clear_if_idle_with(|| {
            on_enter();
            let mut seed = self.lock_seed();
            let previous_seed = seed.master_seed;
            let previous_generation = seed.generation;
            let next_generation = previous_generation.checked_add(1).ok_or(
                RandomServiceError::SeedGenerationExhausted {
                    generation: previous_generation,
                },
            )?;
            seed.master_seed = master_seed;
            seed.generation = next_generation;
            Ok(RandomSeedReceipt::try_new(
                previous_seed,
                master_seed,
                previous_generation,
                next_generation,
            )?)
        }) {
            Ok(result) => result,
            Err(active_leases) => Err(RandomServiceError::ReseedBlocked { active_leases }),
        }
    }

    fn lock_seed(&self) -> MutexGuard<'_, RandomSeedAuthority> {
        self.seed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
