use std::time::Duration;

use thiserror::Error;

use super::{Fixed, Virtual};

/// Atomically applied configuration for the runtime-owned virtual and fixed clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicy {
    virtual_max_delta: Duration,
    virtual_relative_speed: f64,
    fixed_timestep: Duration,
}

impl Default for TimePolicy {
    fn default() -> Self {
        Self::new(
            Virtual::default().max_delta(),
            Virtual::default().relative_speed_f64(),
            Fixed::default().timestep(),
        )
    }
}

impl TimePolicy {
    pub const fn new(
        virtual_max_delta: Duration,
        virtual_relative_speed: f64,
        fixed_timestep: Duration,
    ) -> Self {
        Self {
            virtual_max_delta,
            virtual_relative_speed,
            fixed_timestep,
        }
    }

    pub const fn virtual_max_delta(self) -> Duration {
        self.virtual_max_delta
    }

    pub const fn virtual_relative_speed(self) -> f64 {
        self.virtual_relative_speed
    }

    pub const fn fixed_timestep(self) -> Duration {
        self.fixed_timestep
    }

    pub const fn with_virtual_max_delta(mut self, virtual_max_delta: Duration) -> Self {
        self.virtual_max_delta = virtual_max_delta;
        self
    }

    pub const fn with_virtual_relative_speed(mut self, virtual_relative_speed: f64) -> Self {
        self.virtual_relative_speed = virtual_relative_speed;
        self
    }

    pub const fn with_fixed_timestep(mut self, fixed_timestep: Duration) -> Self {
        self.fixed_timestep = fixed_timestep;
        self
    }

    pub fn validate(self) -> Result<(), TimePolicyError> {
        if self.virtual_max_delta.is_zero() {
            return Err(TimePolicyError::VirtualMaxDeltaZero);
        }
        if !self.virtual_relative_speed.is_finite() {
            return Err(TimePolicyError::VirtualRelativeSpeedNotFinite);
        }
        if self.virtual_relative_speed < 0.0 {
            return Err(TimePolicyError::VirtualRelativeSpeedNegative);
        }
        if self.fixed_timestep.is_zero() {
            return Err(TimePolicyError::FixedTimestepZero);
        }
        Ok(())
    }
}

/// A requested time-policy change that must validate before the runtime mutates its clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicyTransaction {
    requested: TimePolicy,
}

impl TimePolicyTransaction {
    pub const fn new(requested: TimePolicy) -> Self {
        Self { requested }
    }

    pub const fn requested(self) -> TimePolicy {
        self.requested
    }

    pub fn validate(self) -> Result<(), TimePolicyError> {
        self.requested.validate()
    }

    /// Prepares the requested values without mutating a runtime clock authority.
    pub fn prepare(self) -> Result<TimePolicy, TimePolicyError> {
        self.validate()?;
        Ok(self.requested)
    }
}

/// Typed rejection for a requested runtime time policy.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TimePolicyError {
    #[error("virtual time max delta must be non-zero")]
    VirtualMaxDeltaZero,
    #[error("virtual time relative speed must be finite")]
    VirtualRelativeSpeedNotFinite,
    #[error("virtual time relative speed must be non-negative")]
    VirtualRelativeSpeedNegative,
    #[error("fixed timestep must be non-zero")]
    FixedTimestepZero,
    #[error("time policy cannot change while a fixed step is active")]
    FixedStepActive,
    #[error("fixed timestep cannot change while {remaining:?} of fixed debt is pending")]
    FixedStepDebtPending { remaining: Duration },
}
