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
}
