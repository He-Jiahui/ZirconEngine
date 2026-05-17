use std::time::Duration;

use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::{
    CoreRuntime, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC,
    TIME_FRAME_TIME_DIAGNOSTIC,
};

#[test]
fn core_runtime_advances_real_virtual_and_fixed_time_together() {
    let runtime = CoreRuntime::new();

    let advance = runtime.advance_time_by(Duration::from_millis(34), 8);
    let clocks = runtime.time_clocks();

    assert_eq!(advance.real_delta(), Duration::from_millis(34));
    assert_eq!(clocks.real().delta(), Duration::from_millis(34));
    assert_eq!(clocks.real().elapsed(), Duration::from_millis(34));
    assert_eq!(clocks.real().frame_index(), 1);
    assert_eq!(clocks.virtual_time().delta(), Duration::from_millis(34));
    assert_eq!(clocks.virtual_time().elapsed(), Duration::from_millis(34));
    assert_eq!(clocks.virtual_time().frame_index(), 1);
    assert_eq!(advance.fixed_step_plan().step_count, 2);
    assert_eq!(clocks.fixed().frame_index(), 2);
    assert_eq!(clocks.fixed().overstep(), Duration::from_micros(2_750));
}

#[test]
fn core_runtime_virtual_pause_scale_and_clamp_feed_fixed_time() {
    let runtime = CoreRuntime::new();
    runtime.set_fixed_timestep(Duration::from_millis(10));

    runtime.pause_virtual_time();
    let paused = runtime.advance_time_by(Duration::from_millis(35), 8);

    assert_eq!(paused.fixed_step_plan().step_count, 0);
    assert_eq!(runtime.virtual_time().delta(), Duration::ZERO);
    assert_eq!(runtime.fixed_time().overstep(), Duration::ZERO);

    runtime.unpause_virtual_time();
    runtime.set_virtual_time_relative_speed_f64(0.5);
    let scaled = runtime.advance_time_by(Duration::from_millis(40), 8);

    assert_eq!(scaled.fixed_step_plan().step_count, 2);
    assert_eq!(runtime.virtual_time().delta(), Duration::from_millis(20));
    assert_eq!(runtime.fixed_time().elapsed(), Duration::from_millis(20));

    runtime.set_virtual_time_max_delta(Duration::from_millis(5));
    let clamped = runtime.advance_time_by(Duration::from_millis(100), 8);

    assert_eq!(clamped.fixed_step_plan().step_count, 0);
    assert_eq!(runtime.virtual_time().delta(), Duration::from_millis(5));
    assert_eq!(runtime.fixed_time().overstep(), Duration::from_millis(5));
}

#[test]
fn core_runtime_records_bevy_style_time_diagnostics() {
    let runtime = CoreRuntime::new();

    runtime.advance_time_by(Duration::from_millis(20), 8);

    let diagnostics = collect_runtime_diagnostics(&runtime.handle()).store;
    let frame_time = series_value(&diagnostics, TIME_FRAME_TIME_DIAGNOSTIC).unwrap();
    let fps = series_value(&diagnostics, TIME_FPS_DIAGNOSTIC).unwrap();
    let frame_count = series_value(&diagnostics, TIME_FRAME_COUNT_DIAGNOSTIC).unwrap();
    let fixed_steps = series_value(&diagnostics, TIME_FIXED_STEPS_DIAGNOSTIC).unwrap();

    assert_eq!(frame_time, 20.0);
    assert!((fps - 50.0).abs() < 0.000_001);
    assert_eq!(frame_count, 1.0);
    assert_eq!(fixed_steps, 1.0);
}

fn series_value(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}
