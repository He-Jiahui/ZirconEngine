use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::diagnostics::{
    DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore, DiagnosticStoreSnapshot,
};

use super::super::{
    format_diagnostic_store_current_snapshot, format_diagnostic_store_snapshot,
    format_diagnostic_values, DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
};

#[test]
fn diagnostic_series_format_preserves_every_optional_field_combination() {
    struct Case {
        unit: Option<&'static str>,
        smoothed: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
        expected: &'static str,
    }

    let cases = [
        Case {
            unit: None,
            smoothed: None,
            min: None,
            max: None,
            expected: "diag.value: 12.500000",
        },
        Case {
            unit: None,
            smoothed: None,
            min: Some(-4.0),
            max: None,
            expected: "diag.value: 12.500000",
        },
        Case {
            unit: None,
            smoothed: None,
            min: None,
            max: Some(99.75),
            expected: "diag.value: 12.500000",
        },
        Case {
            unit: None,
            smoothed: None,
            min: Some(-4.0),
            max: Some(99.75),
            expected: "diag.value: 12.500000",
        },
        Case {
            unit: None,
            smoothed: Some(10.25),
            min: None,
            max: None,
            expected: "diag.value: 12.500000 (smoothed 10.250000)",
        },
        Case {
            unit: None,
            smoothed: Some(10.25),
            min: Some(-4.0),
            max: None,
            expected: "diag.value: 12.500000 (smoothed 10.250000, min -4.000000)",
        },
        Case {
            unit: None,
            smoothed: Some(10.25),
            min: None,
            max: Some(99.75),
            expected: "diag.value: 12.500000 (smoothed 10.250000, max 99.750000)",
        },
        Case {
            unit: None,
            smoothed: Some(10.25),
            min: Some(-4.0),
            max: Some(99.75),
            expected: "diag.value: 12.500000 (smoothed 10.250000, min -4.000000, max 99.750000)",
        },
        Case {
            unit: Some("ms"),
            smoothed: None,
            min: None,
            max: None,
            expected: "diag.value: 12.500000ms",
        },
        Case {
            unit: Some("ms"),
            smoothed: None,
            min: Some(-4.0),
            max: None,
            expected: "diag.value: 12.500000ms",
        },
        Case {
            unit: Some("ms"),
            smoothed: None,
            min: None,
            max: Some(99.75),
            expected: "diag.value: 12.500000ms",
        },
        Case {
            unit: Some("ms"),
            smoothed: None,
            min: Some(-4.0),
            max: Some(99.75),
            expected: "diag.value: 12.500000ms",
        },
        Case {
            unit: Some("ms"),
            smoothed: Some(10.25),
            min: None,
            max: None,
            expected: "diag.value: 12.500000ms (smoothed 10.250000ms)",
        },
        Case {
            unit: Some("ms"),
            smoothed: Some(10.25),
            min: Some(-4.0),
            max: None,
            expected: "diag.value: 12.500000ms (smoothed 10.250000ms, min -4.000000ms)",
        },
        Case {
            unit: Some("ms"),
            smoothed: Some(10.25),
            min: None,
            max: Some(99.75),
            expected: "diag.value: 12.500000ms (smoothed 10.250000ms, max 99.750000ms)",
        },
        Case {
            unit: Some("ms"),
            smoothed: Some(10.25),
            min: Some(-4.0),
            max: Some(99.75),
            expected:
                "diag.value: 12.500000ms (smoothed 10.250000ms, min -4.000000ms, max 99.750000ms)",
        },
    ];

    for case in cases {
        assert_eq!(
            format_diagnostic_values(
                "diag.value",
                12.5,
                case.unit,
                case.smoothed,
                case.min,
                case.max,
            ),
            case.expected
        );
    }
}

#[test]
fn diagnostic_store_snapshot_omits_series_without_a_current_value() {
    let snapshot = DiagnosticStoreSnapshot {
        series: vec![DiagnosticSeriesSnapshot {
            path: DiagnosticPath::new("diag.empty"),
            unit: Some("ms".to_owned()),
            subsystem_tags: vec!["diagnostics".to_owned()],
            current: None,
            smoothed: Some(10.25),
            min: Some(-4.0),
            max: Some(99.75),
            history: Vec::new(),
        }],
    };

    assert!(format_diagnostic_store_snapshot(&snapshot).is_empty());
}

#[test]
fn diagnostic_series_formatter_uses_one_output_buffer_without_temporary_formats() {
    let source = include_str!("../../diagnostics.rs");
    let start = source
        .find("fn format_diagnostic_values(")
        .expect("diagnostic value formatter");
    let end = source[start..]
        .find("#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("diagnostic value formatter end");
    let formatter = &source[start..end];

    assert!(
        formatter.contains("String::with_capacity("),
        "the formatter should reserve its single output buffer"
    );
    assert!(
        formatter.contains("write!(&mut line"),
        "all numeric fields should write directly into the output buffer"
    );
    assert!(
        !formatter.contains("format!("),
        "temporary formatted Strings reintroduce per-field allocations"
    );
}

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
#[ignore = "performance evidence; managed Windows Release requires candidate p95 <= 95% of legacy"]
fn diagnostic_series_single_buffer_release_percentiles() {
    const SAMPLE_PAIRS: usize = 31;
    const FORMATS_PER_SAMPLE: usize = 4_096;
    const MAX_CANDIDATE_P95_PERCENT: u128 = 95;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_checksum = 0_usize;
    let mut candidate_checksum = 0_usize;

    black_box(measure_diagnostic_formatter(
        legacy_format_diagnostic_values,
        128,
    ));
    black_box(measure_diagnostic_formatter(format_diagnostic_values, 128));

    for pair in 0..SAMPLE_PAIRS {
        let (legacy, candidate) = if pair % 2 == 0 {
            (
                measure_diagnostic_formatter(legacy_format_diagnostic_values, FORMATS_PER_SAMPLE),
                measure_diagnostic_formatter(format_diagnostic_values, FORMATS_PER_SAMPLE),
            )
        } else {
            let candidate =
                measure_diagnostic_formatter(format_diagnostic_values, FORMATS_PER_SAMPLE);
            let legacy =
                measure_diagnostic_formatter(legacy_format_diagnostic_values, FORMATS_PER_SAMPLE);
            (legacy, candidate)
        };
        legacy_samples.push(legacy.0);
        candidate_samples.push(candidate.0);
        legacy_checksum = legacy_checksum.wrapping_add(legacy.1);
        candidate_checksum = candidate_checksum.wrapping_add(candidate.1);
    }

    assert_eq!(candidate_checksum, legacy_checksum);
    let legacy_p50_ns = nearest_rank_percentile(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let legacy_p99_ns = nearest_rank_percentile(&legacy_samples, 99);
    let candidate_p50_ns = nearest_rank_percentile(&candidate_samples, 50);
    let candidate_p95_ns = nearest_rank_percentile(&candidate_samples, 95);
    let candidate_p99_ns = nearest_rank_percentile(&candidate_samples, 99);

    println!(
        "RUNTIME_DIAGNOSTIC_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} formats_per_sample={FORMATS_PER_SAMPLE} pair_order=alternating_legacy_first_even percentile_method=nearest_rank legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} candidate_p50_ns={candidate_p50_ns} candidate_p95_ns={candidate_p95_ns} candidate_p99_ns={candidate_p99_ns} p95_limit_percent={MAX_CANDIDATE_P95_PERCENT} legacy_raw_ns={legacy_samples:?} candidate_raw_ns={candidate_samples:?}"
    );
    assert!(
        candidate_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(MAX_CANDIDATE_P95_PERCENT),
        "single-buffer formatter p95 must be at most {MAX_CANDIDATE_P95_PERCENT}% of legacy: legacy={legacy_p95_ns}ns candidate={candidate_p95_ns}ns"
    );
}

type DiagnosticFormatter =
    fn(&str, f64, Option<&str>, Option<f64>, Option<f64>, Option<f64>) -> String;

fn measure_diagnostic_formatter(
    formatter: DiagnosticFormatter,
    iterations: usize,
) -> (u128, usize) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for index in 0..iterations {
        let value = index as f64 + 0.125;
        let line = formatter(
            black_box("render.frame.diagnostic"),
            black_box(value),
            black_box(Some("milliseconds")),
            black_box(Some(value * 0.95)),
            black_box(Some(-value)),
            black_box(Some(value * 1.25)),
        );
        checksum = checksum.wrapping_add(black_box(line).len());
    }
    (started.elapsed().as_nanos(), checksum)
}

fn legacy_format_diagnostic_values(
    path: &str,
    current: f64,
    unit: Option<&str>,
    smoothed: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
) -> String {
    let unit = unit.unwrap_or("");
    let mut line = format!("{path}: {current:.6}{unit}");
    if let Some(smoothed) = smoothed {
        line.push_str(&format!(" (smoothed {smoothed:.6}{unit}"));
        if let Some(min) = min {
            line.push_str(&format!(", min {min:.6}{unit}"));
        }
        if let Some(max) = max {
            line.push_str(&format!(", max {max:.6}{unit}"));
        }
        line.push(')');
    }
    line
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
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
