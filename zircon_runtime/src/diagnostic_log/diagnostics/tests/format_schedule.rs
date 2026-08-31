use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;

use super::super::{
    format_diagnostic_store_current_snapshot, format_diagnostic_store_snapshot,
    DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
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
fn diagnostic_store_current_snapshot_preserves_log_output() {
    let mut store = DiagnosticStore::new(4);
    store.record("time.frame_time", 1, 20.0, Some("ms"), ["time", "frame"]);
    store.record("time.frame_time", 2, 30.0, Some("ms"), ["time", "frame"]);

    assert_eq!(
        format_diagnostic_store_current_snapshot(&store.current_snapshot()),
        format_diagnostic_store_snapshot(&store.snapshot())
    );
}

#[test]
fn runtime44_batch_schedule_repeats_after_wait_duration() {
    let mut schedule = DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT);

    assert!(schedule.is_enabled());
    assert_eq!(schedule.wait_duration(), Duration::from_secs(1));
    assert!(!schedule.tick(Duration::from_millis(400)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(400));
    assert!(!schedule.tick(Duration::from_millis(500)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(900));
    assert!(schedule.tick(Duration::from_millis(150)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(50));
    assert_eq!(schedule.last_periods_due(), 1);
    assert_eq!(schedule.coalesced_periods(), 0);
}

#[test]
fn runtime44_batch_schedule_reports_coalesced_periods_and_preserves_remainder() {
    let mut schedule = DiagnosticStoreLogSchedule::repeating(Duration::from_secs(1));

    assert!(!schedule.tick(Duration::from_millis(900)));
    assert_eq!(schedule.last_periods_due(), 0);
    assert!(schedule.tick(Duration::from_millis(5_250)));
    assert_eq!(schedule.elapsed(), Duration::from_millis(150));
    assert_eq!(schedule.last_periods_due(), 6);
    assert_eq!(schedule.coalesced_periods(), 5);

    assert!(schedule.tick(Duration::from_millis(2_850)));
    assert_eq!(schedule.elapsed(), Duration::ZERO);
    assert_eq!(schedule.last_periods_due(), 3);
    assert_eq!(schedule.coalesced_periods(), 7);
}

#[test]
fn runtime44_batch_schedule_saturates_large_period_counts() {
    let mut schedule = DiagnosticStoreLogSchedule::repeating(Duration::from_nanos(1));

    assert!(schedule.tick(Duration::MAX));
    assert_eq!(schedule.elapsed(), Duration::ZERO);
    assert_eq!(schedule.last_periods_due(), u64::MAX);
    assert_eq!(schedule.coalesced_periods(), u64::MAX);

    assert!(schedule.tick(Duration::MAX));
    assert_eq!(schedule.last_periods_due(), u64::MAX);
    assert_eq!(schedule.coalesced_periods(), u64::MAX);
}

#[test]
fn runtime44_batch_schedule_can_be_disabled_or_every_tick() {
    let mut disabled = DiagnosticStoreLogSchedule::disabled();
    let mut every_tick = DiagnosticStoreLogSchedule::repeating(Duration::ZERO);

    assert!(!disabled.is_enabled());
    assert!(!disabled.tick(Duration::from_secs(10)));
    assert_eq!(disabled.last_periods_due(), 0);
    assert!(every_tick.tick(Duration::ZERO));
    assert_eq!(every_tick.last_periods_due(), 1);
    assert!(every_tick.tick(Duration::from_millis(16)));
    assert_eq!(every_tick.elapsed(), Duration::ZERO);
    assert_eq!(every_tick.last_periods_due(), 1);
    assert_eq!(every_tick.coalesced_periods(), 0);
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn runtime44_batch_schedule_large_delta_evidence() {
    const DAYS: u64 = 365;
    const MILLIS_PER_DAY: u64 = 24 * 60 * 60 * 1_000;
    const MAX_ELAPSED: Duration = Duration::from_secs(2);

    let wait = Duration::from_millis(1);
    let delta = Duration::from_millis(DAYS * MILLIS_PER_DAY);
    let legacy_period_reductions = delta.as_millis() as u64;
    let mut schedule = DiagnosticStoreLogSchedule::repeating(wait);
    let started = Instant::now();

    assert!(schedule.tick(delta));

    let elapsed = started.elapsed();
    let optimized_division_steps = 1_u64;
    let reduction_basis_points = ((legacy_period_reductions - optimized_division_steps) as u128
        * 10_000
        / legacy_period_reductions as u128) as u64;
    assert_eq!(schedule.elapsed(), Duration::ZERO);
    assert_eq!(schedule.last_periods_due(), legacy_period_reductions);
    assert_eq!(schedule.coalesced_periods(), legacy_period_reductions - 1);
    assert!(elapsed <= MAX_ELAPSED, "large-delta tick took {elapsed:?}");
    println!(
        "RUNTIME_DIAGNOSTIC_SCHEDULE_BENCH_V1 legacy_period_reductions={} optimized_division_steps={} reduction_basis_points={} elapsed_ns={}",
        legacy_period_reductions,
        optimized_division_steps,
        reduction_basis_points,
        elapsed.as_nanos()
    );
}
