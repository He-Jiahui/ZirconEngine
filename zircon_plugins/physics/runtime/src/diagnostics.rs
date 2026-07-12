use std::time::Duration;

use zircon_runtime::core::CoreHandle;

pub const PHYSICS_STEP_DURATION_DIAGNOSTIC_PATH: &str = "physics.step.duration_ms";

pub fn record_physics_step_diagnostic(core: &CoreHandle, frame_index: u64, elapsed: Duration) {
    core.record_diagnostic(
        PHYSICS_STEP_DURATION_DIAGNOSTIC_PATH,
        frame_index,
        elapsed.as_secs_f64() * 1_000.0,
        Some("ms"),
        ["physics", "step"],
    );
}
