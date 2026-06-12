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

    pub fn overstep_fraction(&self) -> f32 {
        if self.timestep.is_zero() {
            return 0.0;
        }
        (self.remaining_overstep.as_secs_f64() / self.timestep.as_secs_f64()).clamp(0.0, 1.0) as f32
    }
}
