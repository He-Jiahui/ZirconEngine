use std::time::Duration;

use crate::core::framework::time::{Fixed, FixedStepPlan, Real, Time, Virtual};

/// Diagnostic path for total real-time frames advanced by the runtime.
pub const TIME_FRAME_COUNT_DIAGNOSTIC: &str = "time.frame_count";
/// Diagnostic path for fixed simulation steps drained during the frame.
pub const TIME_FIXED_STEPS_DIAGNOSTIC: &str = "time.fixed_steps";
/// Diagnostic path for real frame duration measured in milliseconds.
pub const TIME_FRAME_TIME_DIAGNOSTIC: &str = "time.frame_time";
/// Diagnostic path for frames per second derived from real frame duration.
pub const TIME_FPS_DIAGNOSTIC: &str = "time.fps";

/// Runtime-owned clock bundle advanced once per outer frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeTimeClocks {
    real: Time<Real>,
    virtual_time: Time<Virtual>,
    fixed: Time<Fixed>,
}

impl Default for RuntimeTimeClocks {
    fn default() -> Self {
        Self {
            real: Time::<Real>::default(),
            virtual_time: Time::<Virtual>::default(),
            fixed: Time::<Fixed>::default(),
        }
    }
}

impl RuntimeTimeClocks {
    pub fn real(&self) -> Time<Real> {
        self.real
    }

    pub fn virtual_time(&self) -> Time<Virtual> {
        self.virtual_time
    }

    pub fn fixed(&self) -> Time<Fixed> {
        self.fixed
    }

    pub fn advance_by(&mut self, real_delta: Duration, max_fixed_steps: u32) -> RuntimeTimeAdvance {
        self.real.advance_by(real_delta);
        self.virtual_time.advance_from_real_delta(real_delta);
        self.fixed.accumulate_overstep(self.virtual_time.delta());
        let fixed_step_plan = self.fixed.drain_steps(max_fixed_steps);

        RuntimeTimeAdvance {
            real_delta,
            fixed_step_plan,
        }
    }

    pub fn pause_virtual_time(&mut self) {
        self.virtual_time.pause();
    }

    pub fn unpause_virtual_time(&mut self) {
        self.virtual_time.unpause();
    }

    pub fn set_virtual_time_max_delta(&mut self, max_delta: Duration) {
        self.virtual_time.set_max_delta(max_delta);
    }

    pub fn set_virtual_time_relative_speed_f64(&mut self, speed: f64) {
        self.virtual_time.set_relative_speed_f64(speed);
    }

    pub fn set_fixed_timestep(&mut self, timestep: Duration) {
        self.fixed.set_timestep(timestep);
    }
}

/// Summary of one runtime time update, including the fixed-step work budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeTimeAdvance {
    real_delta: Duration,
    fixed_step_plan: FixedStepPlan,
}

impl RuntimeTimeAdvance {
    pub fn real_delta(&self) -> Duration {
        self.real_delta
    }

    pub fn fixed_step_plan(&self) -> FixedStepPlan {
        self.fixed_step_plan
    }
}
