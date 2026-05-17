use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::time::{Fixed, Real, Time, Virtual};
use crate::core::time::{
    RuntimeTimeAdvance, RuntimeTimeClocks, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};

use super::CoreHandle;

impl CoreHandle {
    pub fn time_clocks(&self) -> RuntimeTimeClocks {
        *self.inner.time.lock().unwrap()
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
            let mut time = self.inner.time.lock().unwrap();
            let advance = time.advance_by(real_delta, max_fixed_steps);
            (advance, *time)
        };
        record_time_diagnostics(&self.inner.diagnostics, clocks, advance);
        advance
    }

    pub fn tick_time(&self, max_fixed_steps: u32) -> RuntimeTimeAdvance {
        let real_delta = self.inner.frame_clock.lock().unwrap().tick();
        self.advance_time_by(real_delta, max_fixed_steps)
    }

    pub fn pause_virtual_time(&self) {
        self.inner.time.lock().unwrap().pause_virtual_time();
    }

    pub fn unpause_virtual_time(&self) {
        self.inner.time.lock().unwrap().unpause_virtual_time();
    }

    pub fn set_virtual_time_max_delta(&self, max_delta: Duration) {
        self.inner
            .time
            .lock()
            .unwrap()
            .set_virtual_time_max_delta(max_delta);
    }

    pub fn set_virtual_time_relative_speed_f64(&self, speed: f64) {
        self.inner
            .time
            .lock()
            .unwrap()
            .set_virtual_time_relative_speed_f64(speed);
    }

    pub fn set_fixed_timestep(&self, timestep: Duration) {
        self.inner.time.lock().unwrap().set_fixed_timestep(timestep);
    }
}

fn record_time_diagnostics(
    diagnostics: &std::sync::Mutex<DiagnosticStore>,
    clocks: RuntimeTimeClocks,
    advance: RuntimeTimeAdvance,
) {
    let frame_index = clocks.real().frame_index();
    let fixed_steps = advance.fixed_step_plan().step_count as f64;
    let real_delta_seconds = advance.real_delta().as_secs_f64();
    let mut diagnostics = diagnostics.lock().unwrap();

    diagnostics.record(
        TIME_FRAME_COUNT_DIAGNOSTIC,
        frame_index,
        frame_index as f64,
        Some("frame"),
        ["time", "frame"],
    );
    diagnostics.record(
        TIME_FIXED_STEPS_DIAGNOSTIC,
        frame_index,
        fixed_steps,
        Some("step"),
        ["time", "fixed"],
    );
    if real_delta_seconds == 0.0 {
        return;
    }
    diagnostics.record(
        TIME_FRAME_TIME_DIAGNOSTIC,
        frame_index,
        real_delta_seconds * 1_000.0,
        Some("ms"),
        ["time", "frame"],
    );
    diagnostics.record(
        TIME_FPS_DIAGNOSTIC,
        frame_index,
        1.0 / real_delta_seconds,
        Some("hz"),
        ["time", "frame"],
    );
}
