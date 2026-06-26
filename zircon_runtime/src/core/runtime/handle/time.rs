use std::sync::MutexGuard;
use std::time::Duration;

use crate::core::framework::time::{Fixed, Real, Time, Virtual};

use super::super::frame_clock::FrameClock;
use super::super::time::{
    RuntimeTimeAdvance, RuntimeTimeClocks, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
use super::CoreHandle;

impl CoreHandle {
    pub fn time_clocks(&self) -> RuntimeTimeClocks {
        *self.lock_time()
    }

    pub fn real_time(&self) -> Time<Real> {
        self.time_clocks().real()
    }

    pub fn virtual_time(&self) -> Time<Virtual> {
        self.time_clocks().virtual_time()
    }

    pub fn fixed_time(&self) -> Time<Fixed> {
        self.time_clocks().fixed()
    }

    pub fn advance_time_by(
        &self,
        real_delta: Duration,
        max_fixed_steps: u32,
    ) -> RuntimeTimeAdvance {
        let (advance, clocks) = {
            let mut time = self.lock_time();
            let advance = time.advance_by(real_delta, max_fixed_steps);
            (advance, *time)
        };
        record_time_diagnostics(self, clocks, advance);
        advance
    }

    pub fn tick_time(&self, max_fixed_steps: u32) -> RuntimeTimeAdvance {
        let real_delta = self.lock_frame_clock().tick();
        self.advance_time_by(real_delta, max_fixed_steps)
    }

    pub fn pause_virtual_time(&self) {
        self.lock_time().pause_virtual_time();
    }

    pub fn unpause_virtual_time(&self) {
        self.lock_time().unpause_virtual_time();
    }

    pub fn set_virtual_time_max_delta(&self, max_delta: Duration) {
        self.lock_time().set_virtual_time_max_delta(max_delta);
    }

    pub fn set_virtual_time_relative_speed_f64(&self, speed: f64) {
        self.lock_time().set_virtual_time_relative_speed_f64(speed);
    }

    pub fn set_fixed_timestep(&self, timestep: Duration) {
        self.lock_time().set_fixed_timestep(timestep);
    }

    fn lock_time(&self) -> MutexGuard<'_, RuntimeTimeClocks> {
        self.inner
            .time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_frame_clock(&self) -> MutexGuard<'_, FrameClock> {
        self.inner
            .frame_clock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn record_time_diagnostics(
    handle: &CoreHandle,
    clocks: RuntimeTimeClocks,
    advance: RuntimeTimeAdvance,
) {
    let frame_index = clocks.real().frame_index();
    let fixed_steps = advance.fixed_step_plan().step_count as f64;
    let real_delta_seconds = advance.real_delta().as_secs_f64();

    handle.record_diagnostic(
        TIME_FRAME_COUNT_DIAGNOSTIC,
        frame_index,
        frame_index as f64,
        Some("frame"),
        ["time", "frame"],
    );
    handle.record_diagnostic(
        TIME_FIXED_STEPS_DIAGNOSTIC,
        frame_index,
        fixed_steps,
        Some("step"),
        ["time", "fixed"],
    );
    if real_delta_seconds == 0.0 {
        return;
    }
    handle.record_diagnostic(
        TIME_FRAME_TIME_DIAGNOSTIC,
        frame_index,
        real_delta_seconds * 1_000.0,
        Some("ms"),
        ["time", "frame"],
    );
    handle.record_diagnostic(
        TIME_FPS_DIAGNOSTIC,
        frame_index,
        1.0 / real_delta_seconds,
        Some("hz"),
        ["time", "frame"],
    );
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::time::Duration;

    use crate::core::{CoreRuntime, TIME_FRAME_COUNT_DIAGNOSTIC};

    #[test]
    fn core_handle_time_accessors_recover_poisoned_runtime_clocks() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.time.lock().unwrap();
            panic!("poison core handle runtime time");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.frame_clock.lock().unwrap();
            panic!("poison core handle frame clock");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.diagnostics.lock().unwrap();
            panic!("poison core handle time diagnostics");
        }));

        handle.pause_virtual_time();
        assert!(handle.virtual_time().is_paused());
        handle.unpause_virtual_time();
        handle.set_virtual_time_max_delta(Duration::from_millis(33));
        handle.set_virtual_time_relative_speed_f64(1.0);
        handle.set_fixed_timestep(Duration::from_millis(16));

        let advance = handle.advance_time_by(Duration::from_millis(16), 4);
        assert_eq!(advance.real_delta(), Duration::from_millis(16));
        assert_eq!(handle.real_time().frame_index(), 1);
        assert_eq!(handle.fixed_time().timestep(), Duration::from_millis(16));

        let tick_advance = handle.tick_time(4);
        assert!(handle.real_time().frame_index() >= 2);
        assert_eq!(
            tick_advance.real_delta(),
            handle.real_time().delta(),
            "tick_time should advance from the recovered frame clock delta"
        );

        let snapshot = handle.diagnostic_store_snapshot();
        assert!(
            snapshot
                .series
                .iter()
                .any(|series| series.path.as_str() == TIME_FRAME_COUNT_DIAGNOSTIC),
            "time diagnostics should be recorded through the recovered diagnostics store"
        );
    }
}
