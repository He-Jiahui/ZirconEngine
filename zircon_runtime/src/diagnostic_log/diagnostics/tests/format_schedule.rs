use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;

use super::super::{
    format_diagnostic_store_snapshot, DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
};

#[test]
fn diagnostic_store_snapshot_formats_current_smoothed_min_and_max() {
    let mut store = DiagnosticStore::new(4);
    store.record("time.frame_time", 1, 20.0, Some("ms"), ["time", "frame"]);
    store.record("time.frame_time", 2, 30.0, Some("ms"), ["time", "frame"]);

    let lines = format_diagnostic_store_snapshot(&store.snapshot());

    assert_eq!(
        lines,
        vec![
            "time.frame_time: 30.000000ms (smoothed 21.000000ms, min 20.000000ms, max 30.000000ms)"
        ]
    );
}

#[test]
fn diagnostic_store_log_schedule_repeats_after_wait_duration() {
    let mut schedule = DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT);

    assert!(schedule.is_enabled());
    assert_eq!(schedule.wait_duration(), Duration::from_secs(1));
    assert!(!schedule.tick(Duration::from_millis(400)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(400));
    assert!(!schedule.tick(Duration::from_millis(500)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(900));
    assert!(schedule.tick(Duration::from_millis(150)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(50));
}

#[test]
fn diagnostic_store_log_schedule_can_be_disabled_or_every_tick() {
    let mut disabled = DiagnosticStoreLogSchedule::disabled();
    let mut every_tick = DiagnosticStoreLogSchedule::repeating(Duration::ZERO);

    assert!(!disabled.is_enabled());
    assert!(!disabled.tick(Duration::from_secs(10)));
    assert!(every_tick.tick(Duration::ZERO));
    assert!(every_tick.tick(Duration::from_millis(16)));
    assert_eq!(every_tick.elapsed(), Duration::ZERO);
}
