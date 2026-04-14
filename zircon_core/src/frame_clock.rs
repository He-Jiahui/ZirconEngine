//! Frame delta timing.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FrameClock {
    last_tick: Instant,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
        }
    }
}

impl FrameClock {
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        delta
    }
}
