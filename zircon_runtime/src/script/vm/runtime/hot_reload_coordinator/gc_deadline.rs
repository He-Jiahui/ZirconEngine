use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) struct GcFrameDeadline {
    started: Instant,
    budget_micros: u64,
}

#[derive(Debug)]
pub(super) struct GcStepTimer {
    started: Instant,
}

impl GcFrameDeadline {
    pub(super) fn start(budget_micros: u64) -> Self {
        Self {
            started: Instant::now(),
            budget_micros,
        }
    }

    pub(super) fn remaining_micros(&self) -> u64 {
        self.budget_micros
            .saturating_sub(duration_micros(self.started.elapsed()))
    }

    pub(super) fn elapsed_micros(&self) -> u64 {
        duration_micros(self.started.elapsed())
    }

    pub(super) fn begin_step(&self) -> GcStepTimer {
        GcStepTimer {
            started: Instant::now(),
        }
    }
}

impl GcStepTimer {
    pub(super) fn elapsed_micros(&self) -> u64 {
        duration_micros(self.started.elapsed())
    }
}

fn duration_micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}
