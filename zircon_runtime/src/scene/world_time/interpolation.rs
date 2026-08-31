use std::time::Duration;

use super::SimulationTickId;

/// One committed endpoint available to fixed-state interpolation consumers.
///
/// The initial World state is a committed baseline without a simulation tick.
/// It remains observable so a first committed fixed step can interpolate from
/// the initial state without inventing a tick identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedInterpolationState {
    simulation_tick: Option<SimulationTickId>,
    elapsed: Duration,
}

impl FixedInterpolationState {
    pub(crate) const fn new(simulation_tick: Option<SimulationTickId>, elapsed: Duration) -> Self {
        Self {
            simulation_tick,
            elapsed,
        }
    }

    pub const fn simulation_tick(self) -> Option<SimulationTickId> {
        self.simulation_tick
    }

    pub const fn elapsed(self) -> Duration {
        self.elapsed
    }
}

/// Immutable interpolation evidence derived from committed World-local fixed state.
///
/// `fraction` is the residual fixed debt modulo one timestep. It never exposes
/// a begun step and stays bounded even when the fixed-step admission cap leaves
/// multiple whole steps pending.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedInterpolationContext {
    previous: FixedInterpolationState,
    current: FixedInterpolationState,
    remaining_debt: Duration,
    timestep: Duration,
    fraction: f32,
}

impl FixedInterpolationContext {
    pub(crate) const fn new(
        previous: FixedInterpolationState,
        current: FixedInterpolationState,
        remaining_debt: Duration,
        timestep: Duration,
        fraction: f32,
    ) -> Self {
        Self {
            previous,
            current,
            remaining_debt,
            timestep,
            fraction,
        }
    }

    pub const fn previous(self) -> FixedInterpolationState {
        self.previous
    }

    pub const fn current(self) -> FixedInterpolationState {
        self.current
    }

    pub const fn remaining_debt(self) -> Duration {
        self.remaining_debt
    }

    pub const fn timestep(self) -> Duration {
        self.timestep
    }

    pub const fn fraction(self) -> f32 {
        self.fraction
    }
}
