//! 每帧 GC 截止以宿主墙钟为准，后端报告仅作遥测；连续切片必须在剩余预算耗尽时停止。
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) struct GcFrameDeadline {
    started: Instant,
    budget_micros: u64,
    #[cfg(test)]
    test_clock: Option<Arc<GcTestClock>>,
}

#[derive(Debug)]
pub(super) struct GcStepTimer {
    started: Instant,
    #[cfg(test)]
    test_clock: Option<Arc<GcTestClock>>,
    #[cfg(test)]
    test_started_micros: u64,
}

#[cfg(test)]
#[derive(Debug)]
struct GcTestClock {
    elapsed_micros: AtomicU64,
    step_elapsed_micros: u64,
}

impl GcFrameDeadline {
    pub(super) fn start(budget_micros: u64) -> Self {
        Self {
            started: Instant::now(),
            budget_micros,
            #[cfg(test)]
            test_clock: None,
        }
    }

    #[cfg(test)]
    pub(super) fn start_with_test_clock(budget_micros: u64, step_elapsed_micros: u64) -> Self {
        Self {
            started: Instant::now(),
            budget_micros,
            test_clock: Some(Arc::new(GcTestClock {
                elapsed_micros: AtomicU64::new(0),
                step_elapsed_micros,
            })),
        }
    }

    pub(super) fn remaining_micros(&self) -> u64 {
        self.budget_micros.saturating_sub(self.elapsed_micros())
    }

    pub(super) fn elapsed_micros(&self) -> u64 {
        #[cfg(test)]
        if let Some(clock) = &self.test_clock {
            return clock.elapsed_micros.load(Ordering::SeqCst);
        }
        duration_micros(self.started.elapsed())
    }

    pub(super) fn begin_step(&self) -> GcStepTimer {
        GcStepTimer {
            started: Instant::now(),
            #[cfg(test)]
            test_started_micros: self
                .test_clock
                .as_ref()
                .map_or(0, |clock| clock.elapsed_micros.load(Ordering::SeqCst)),
            #[cfg(test)]
            test_clock: self.test_clock.clone(),
        }
    }
}

impl GcStepTimer {
    pub(super) fn elapsed_micros(&self) -> u64 {
        #[cfg(test)]
        if let Some(clock) = &self.test_clock {
            let elapsed = clock
                .elapsed_micros
                .fetch_add(clock.step_elapsed_micros, Ordering::SeqCst)
                .saturating_add(clock.step_elapsed_micros);
            return elapsed.saturating_sub(self.test_started_micros);
        }
        duration_micros(self.started.elapsed())
    }
}

fn duration_micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}
