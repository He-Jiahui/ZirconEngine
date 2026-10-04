use std::time::Duration;

use super::HeadlessHostError;

pub(super) struct HeadlessSchedule {
    timestep: Duration,
    next_tick: Duration,
    last_observed: Duration,
    overruns: u64,
}

impl HeadlessSchedule {
    pub(super) fn new(timestep: Duration) -> Result<Self, HeadlessHostError> {
        if timestep.is_zero() {
            return Err(HeadlessHostError::Clock(
                "headless timestep must be positive",
            ));
        }
        Ok(Self {
            timestep,
            next_tick: Duration::ZERO,
            last_observed: Duration::ZERO,
            overruns: 0,
        })
    }

    pub(super) fn next_tick(&self) -> Duration {
        self.next_tick
    }

    pub(super) fn overruns(&self) -> u64 {
        self.overruns
    }

    pub(super) fn tick_completed(&mut self, now: Duration) -> Result<(), HeadlessHostError> {
        if now < self.last_observed {
            return Err(HeadlessHostError::Clock("headless clock moved backwards"));
        }
        let next = self
            .next_tick
            .checked_add(self.timestep)
            .ok_or(HeadlessHostError::Clock("headless deadline overflow"))?;
        self.next_tick = if next <= now {
            self.overruns = self.overruns.saturating_add(1);
            now.checked_add(self.timestep)
                .ok_or(HeadlessHostError::Clock("headless deadline overflow"))?
        } else {
            next
        };
        self.last_observed = now;
        Ok(())
    }
}
