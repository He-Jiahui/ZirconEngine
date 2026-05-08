use std::marker::PhantomData;
use std::time::Duration;

use super::{Fixed, FixedStepPlan, Virtual};

/// Generic clock state shared by real, virtual, fixed, and future custom clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time<T = ()> {
    context: T,
    delta: Duration,
    elapsed: Duration,
    frame_index: u64,
    marker: PhantomData<T>,
}

impl<T: Default> Default for Time<T> {
    fn default() -> Self {
        Self::new_with(T::default())
    }
}

impl<T> Time<T> {
    pub fn new_with(context: T) -> Self {
        Self {
            context,
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            frame_index: 0,
            marker: PhantomData,
        }
    }

    pub fn context(&self) -> &T {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut T {
        &mut self.context
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_secs_f64(&self) -> f64 {
        self.delta.as_secs_f64()
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn elapsed_secs_f64(&self) -> f64 {
        self.elapsed.as_secs_f64()
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed = self.elapsed.saturating_add(delta);
        self.frame_index = self.frame_index.saturating_add(1);
    }
}

impl Time<Virtual> {
    pub fn advance_from_real_delta(&mut self, real_delta: Duration) {
        let effective_speed = self.context.effective_speed_for_next_delta();
        let scaled = real_delta
            .mul_f64(effective_speed)
            .min(self.context.max_delta());
        self.advance_by(scaled);
    }

    pub fn max_delta(&self) -> Duration {
        self.context.max_delta()
    }

    pub fn set_max_delta(&mut self, max_delta: Duration) {
        self.context.set_max_delta(max_delta);
    }

    pub fn is_paused(&self) -> bool {
        self.context.is_paused()
    }

    pub fn pause(&mut self) {
        self.context.pause();
    }

    pub fn unpause(&mut self) {
        self.context.unpause();
    }

    pub fn relative_speed_f64(&self) -> f64 {
        self.context.relative_speed_f64()
    }

    pub fn set_relative_speed_f64(&mut self, speed: f64) {
        self.context.set_relative_speed_f64(speed);
    }

    pub fn effective_speed_f64(&self) -> f64 {
        self.context.effective_speed_f64()
    }
}

impl Time<Fixed> {
    pub fn from_duration(timestep: Duration) -> Self {
        let mut time = Self::default();
        time.set_timestep(timestep);
        time
    }

    pub fn timestep(&self) -> Duration {
        self.context.timestep()
    }

    pub fn set_timestep(&mut self, timestep: Duration) {
        self.context.set_timestep(timestep);
    }

    pub fn overstep(&self) -> Duration {
        self.context.overstep()
    }

    pub fn accumulate_overstep(&mut self, delta: Duration) {
        self.context.accumulate_overstep(delta);
    }

    pub fn drain_steps(&mut self, max_steps: u32) -> FixedStepPlan {
        let mut step_count = 0;
        let timestep = self.timestep();
        while step_count < max_steps && self.context.take_step() {
            self.advance_by(timestep);
            step_count += 1;
        }
        FixedStepPlan::new(
            step_count,
            timestep,
            timestep.saturating_mul(step_count),
            self.overstep(),
        )
    }
}
