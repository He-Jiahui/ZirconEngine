use std::fmt;
use std::time::Duration;

use thiserror::Error;

use crate::core::CoreError;
use crate::scene::world_time::WorldTimeAdvanceError;
use crate::scene::{SimulationTickId, SystemStage};

/// Fixed-step operation that rejected one simulation tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedStepFailurePhase {
    Stage(SystemStage),
    Commit,
}

impl fmt::Display for FixedStepFailurePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stage(stage) => write!(formatter, "stage {stage:?}"),
            Self::Commit => formatter.write_str("commit"),
        }
    }
}

/// Stable clock/debt evidence captured after a fixed-step abort completes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedStepFailureReceipt {
    phase: FixedStepFailurePhase,
    tick: SimulationTickId,
    system_id: Option<String>,
    committed_steps: u32,
    remaining_debt: Duration,
    observed_world_generation: u64,
}

impl fmt::Display for FixedStepFailureReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "fixed step {:?} failed during {}",
            self.tick, self.phase
        )
    }
}

impl FixedStepFailureReceipt {
    pub(crate) fn new(
        phase: FixedStepFailurePhase,
        tick: SimulationTickId,
        system_id: Option<String>,
        committed_steps: u32,
        remaining_debt: Duration,
        observed_world_generation: u64,
    ) -> Self {
        Self {
            phase,
            tick,
            system_id,
            committed_steps,
            remaining_debt,
            observed_world_generation,
        }
    }

    pub const fn phase(&self) -> FixedStepFailurePhase {
        self.phase
    }

    pub const fn tick(&self) -> SimulationTickId {
        self.tick
    }

    pub fn system_id(&self) -> Option<&str> {
        self.system_id.as_deref()
    }

    pub const fn committed_steps(&self) -> u32 {
        self.committed_steps
    }

    pub const fn remaining_debt(&self) -> Duration {
        self.remaining_debt
    }

    pub const fn observed_world_generation(&self) -> u64 {
        self.observed_world_generation
    }
}

/// Failure returned by one Level tick.
#[derive(Debug, Error)]
pub enum LevelTickError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    WorldTime(#[from] WorldTimeAdvanceError),
    #[error("{receipt}: {source}")]
    FixedStep {
        receipt: FixedStepFailureReceipt,
        #[source]
        source: CoreError,
    },
}

impl LevelTickError {
    pub(crate) fn fixed_step(receipt: FixedStepFailureReceipt, source: CoreError) -> Self {
        Self::FixedStep { receipt, source }
    }

    pub const fn fixed_step_receipt(&self) -> Option<&FixedStepFailureReceipt> {
        match self {
            Self::Core(_) | Self::WorldTime(_) => None,
            Self::FixedStep { receipt, .. } => Some(receipt),
        }
    }

    pub const fn world_time_advance_error(&self) -> Option<&WorldTimeAdvanceError> {
        match self {
            Self::WorldTime(source) => Some(source),
            Self::Core(_) | Self::FixedStep { .. } => None,
        }
    }
}
