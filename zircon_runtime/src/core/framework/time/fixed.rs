use std::time::Duration;

/// Fixed-step clock marker and accumulator state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fixed {
    timestep: Duration,
    overstep: Duration,
}

impl Default for Fixed {
    fn default() -> Self {
        Self {
            timestep: Duration::from_micros(15_625),
            overstep: Duration::ZERO,
        }
    }
}

impl Fixed {
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    pub fn set_timestep(&mut self, timestep: Duration) {
        assert_ne!(timestep, Duration::ZERO, "fixed timestep must be non-zero");
        self.timestep = timestep;
    }

    pub fn overstep(&self) -> Duration {
        self.overstep
    }

    pub fn accumulate_overstep(&mut self, delta: Duration) {
        self.overstep = self.overstep.saturating_add(delta);
    }

    pub(crate) fn take_step(&mut self) -> bool {
        let Some(remaining) = self.overstep.checked_sub(self.timestep) else {
            return false;
        };
        self.overstep = remaining;
        true
    }
}
