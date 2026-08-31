use std::marker::PhantomData;
use std::time::Duration;

use super::{ClockDomainMarker, ClockDomainStamp, Fixed, FixedStepPlan, MonotonicReal, Virtual};

/// Read-only clock observation shared by the engine-owned clock domains.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time<T: ClockDomainMarker = MonotonicReal> {
    context: T,
    delta: Duration,
    elapsed: Duration,
    frame_index: u64,
    clock_domain_stamp: ClockDomainStamp,
    marker: PhantomData<T>,
}

impl<T: ClockDomainMarker + Default> Default for Time<T> {
    fn default() -> Self {
        Self::new_with(T::default())
    }
}

impl<T: ClockDomainMarker> Time<T> {
    pub fn new_with(context: T) -> Self {
        Self {
            context,
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            frame_index: 0,
            clock_domain_stamp: ClockDomainStamp::initial(T::CLOCK_DOMAIN),
            marker: PhantomData,
        }
    }

    pub fn context(&self) -> &T {
        &self.context
    }

    pub(crate) fn context_mut(&mut self) -> &mut T {
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

    pub fn clock_domain_stamp(&self) -> ClockDomainStamp {
        self.clock_domain_stamp
    }

    pub(crate) fn bump_clock_domain_epoch(&mut self) {
        self.clock_domain_stamp.bump_epoch();
    }

    pub(crate) fn set_clock_domain_source_generation(&mut self, source_generation: u64) {
        self.clock_domain_stamp
            .set_source_generation(source_generation);
    }

    pub(crate) fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed = self.elapsed.saturating_add(delta);
        self.frame_index = self.frame_index.saturating_add(1);
    }
}

impl Time<Virtual> {
    pub(crate) fn advance_from_real_delta(&mut self, real_delta: Duration) {
        let effective_speed = self.context.effective_speed_for_next_delta();
        let scaled = scale_virtual_delta(real_delta, effective_speed, self.context.max_delta());
        self.advance_by(scaled);
    }

    pub fn max_delta(&self) -> Duration {
        self.context.max_delta()
    }

    pub(crate) fn set_max_delta(&mut self, max_delta: Duration) {
        self.context.set_max_delta(max_delta);
    }

    pub fn is_paused(&self) -> bool {
        self.context.is_paused()
    }

    pub(crate) fn pause(&mut self) {
        self.context.pause();
    }

    pub(crate) fn unpause(&mut self) {
        self.context.unpause();
    }

    pub fn relative_speed_f64(&self) -> f64 {
        self.context.relative_speed_f64()
    }

    pub(crate) fn set_relative_speed_f64(&mut self, speed: f64) {
        self.context.set_relative_speed_f64(speed);
    }

    pub fn effective_speed_f64(&self) -> f64 {
        self.context.effective_speed_f64()
    }
}

fn scale_virtual_delta(
    real_delta: Duration,
    effective_speed: f64,
    max_delta: Duration,
) -> Duration {
    if real_delta.is_zero() || effective_speed == 0.0 {
        return Duration::ZERO;
    }

    // Apply the clamp before constructing a Duration: `Duration::mul_f64` can
    // panic for a valid finite speed whose scaled duration exceeds its range.
    let scaled_seconds = real_delta.as_secs_f64() * effective_speed;
    if !scaled_seconds.is_finite() || scaled_seconds >= max_delta.as_secs_f64() {
        return max_delta;
    }

    Duration::try_from_secs_f64(scaled_seconds)
        .map(|scaled| scaled.min(max_delta))
        .unwrap_or(max_delta)
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

    pub(crate) fn set_timestep(&mut self, timestep: Duration) {
        self.context.set_timestep(timestep);
    }

    pub fn overstep(&self) -> Duration {
        self.context.overstep()
    }

    pub(crate) fn accumulate_overstep(&mut self, delta: Duration) {
        self.context.accumulate_overstep(delta);
    }

    /// Describes debt that can be consumed this frame without mutating the clock.
    pub(crate) fn plan_steps(&self, max_steps: u32) -> FixedStepPlan {
        let timestep = self.timestep();
        let step_count = self.context.available_steps(max_steps);
        let consumed = timestep.saturating_mul(step_count);
        FixedStepPlan::new(
            step_count,
            timestep,
            consumed,
            self.overstep().saturating_sub(consumed),
        )
    }

    /// Commits one already-begun fixed step if sufficient debt remains.
    pub(crate) fn try_commit_step(&mut self) -> bool {
        if self.context.take_steps(1) == 0 {
            return false;
        }
        let timestep = self.timestep();
        self.advance_by(timestep);
        true
    }

    #[cfg(test)]
    pub(crate) fn drain_steps(&mut self, max_steps: u32) -> FixedStepPlan {
        let plan = self.plan_steps(max_steps);
        let step_count = self.context.take_steps(max_steps);
        if step_count > 0 {
            self.delta = plan.timestep;
            self.elapsed = self.elapsed.saturating_add(plan.consumed);
            self.frame_index = self.frame_index.saturating_add(u64::from(step_count));
        }
        plan
    }
}
