use std::time::Duration;

/// Fixed-step clock marker and accumulator state.
/// World 固定步的欠债累加器；先按外层帧积累虚拟时间，成功提交的步才扣除欠债。
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

    pub(crate) fn set_timestep(&mut self, timestep: Duration) {
        assert_ne!(timestep, Duration::ZERO, "fixed timestep must be non-zero");
        self.timestep = timestep;
    }

    pub fn overstep(&self) -> Duration {
        self.overstep
    }

    pub(crate) fn accumulate_overstep(&mut self, delta: Duration) {
        self.overstep = self.overstep.saturating_add(delta);
    }

    pub(crate) fn available_steps(&self, max_steps: u32) -> u32 {
        (self.overstep.as_nanos() / self.timestep.as_nanos()).min(u128::from(max_steps)) as u32
    }

    pub(crate) fn take_steps(&mut self, max_steps: u32) -> u32 {
        let available_steps = self.available_steps(max_steps);
        let consumed = self.timestep.saturating_mul(available_steps);
        self.overstep = self.overstep.saturating_sub(consumed);
        available_steps
    }
}
