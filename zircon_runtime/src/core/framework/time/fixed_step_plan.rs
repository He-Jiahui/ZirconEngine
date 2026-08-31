use std::time::Duration;

/// Result of draining fixed-timestep overstep for one outer update.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedStepPlan {
    pub step_count: u32,
    pub timestep: Duration,
    pub consumed: Duration,
    pub remaining_overstep: Duration,
}

impl FixedStepPlan {
    pub fn new(
        step_count: u32,
        timestep: Duration,
        consumed: Duration,
        remaining_overstep: Duration,
    ) -> Self {
        Self {
            step_count,
            timestep,
            consumed,
            remaining_overstep,
        }
    }

    /// Total unconsumed fixed-step debt after this outer frame.
    pub fn debt_duration(&self) -> Duration {
        self.remaining_overstep
    }

    /// Number of complete fixed timesteps still owed after this outer frame.
    pub fn debt_whole_steps(&self) -> u128 {
        if self.timestep.is_zero() {
            return 0;
        }
        self.remaining_overstep.as_nanos() / self.timestep.as_nanos()
    }

    /// Unbounded debt measured in fixed timesteps for scheduling and health telemetry.
    pub fn debt_timestep_ratio_f64(&self) -> f64 {
        if self.timestep.is_zero() {
            return 0.0;
        }
        self.remaining_overstep.as_secs_f64() / self.timestep.as_secs_f64()
    }

    /// Fractional remainder of debt that can interpolate between two adjacent fixed states.
    pub fn interpolation_fraction(&self) -> f32 {
        if self.timestep.is_zero() {
            return 0.0;
        }
        let timestep_nanos = self.timestep.as_nanos();
        let remainder_nanos = self.remaining_overstep.as_nanos() % timestep_nanos;
        (remainder_nanos as f64 / timestep_nanos as f64) as f32
    }
}
