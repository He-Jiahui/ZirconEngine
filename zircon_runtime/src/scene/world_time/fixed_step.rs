use std::time::Duration;

use thiserror::Error;

/// Stable identity for one World-local fixed simulation step.
///
/// A tick index is only meaningful together with the World generation and the
/// fixed clock epoch that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimulationTickId {
    world_generation: u64,
    fixed_epoch: u64,
    tick_index: u64,
}

impl SimulationTickId {
    pub(crate) const fn new(world_generation: u64, fixed_epoch: u64, tick_index: u64) -> Self {
        Self {
            world_generation,
            fixed_epoch,
            tick_index,
        }
    }

    pub const fn world_generation(self) -> u64 {
        self.world_generation
    }

    pub const fn fixed_epoch(self) -> u64 {
        self.fixed_epoch
    }

    pub const fn tick_index(self) -> u64 {
        self.tick_index
    }
}

/// A non-cloneable capability for one begun but uncommitted fixed step.
#[derive(Debug)]
pub(crate) struct WorldFixedStep {
    id: SimulationTickId,
    timestep: Duration,
    elapsed: Duration,
}

impl WorldFixedStep {
    pub(crate) const fn new(id: SimulationTickId, timestep: Duration, elapsed: Duration) -> Self {
        Self {
            id,
            timestep,
            elapsed,
        }
    }

    pub const fn id(&self) -> SimulationTickId {
        self.id
    }

    pub const fn timestep(&self) -> Duration {
        self.timestep
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

/// Rejection from a World-local fixed step state transition.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum WorldFixedStepError {
    #[error("fixed step {active:?} is already active")]
    ActiveStep { active: SimulationTickId },
    #[error("no fixed-step budget remains for this outer frame")]
    BudgetExhausted,
    #[error("fixed-step debt is smaller than the requested timestep")]
    InsufficientDebt,
    #[error("fixed step {submitted:?} does not match the active step {active:?}")]
    ActiveStepMismatch {
        active: SimulationTickId,
        submitted: SimulationTickId,
    },
    #[error("fixed step {submitted:?} has no active transaction")]
    NoActiveStep { submitted: SimulationTickId },
    #[error("World generation changed during fixed step: expected {expected}, found {actual}")]
    WorldGenerationChanged { expected: u64, actual: u64 },
}
