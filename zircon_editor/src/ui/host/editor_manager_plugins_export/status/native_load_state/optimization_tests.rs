use std::hint::black_box;
use std::time::{Duration, Instant};

use super::native_load_state_label;

const SAMPLE_COUNT: usize = 17;
const REUSE_ITERATIONS: usize = 512;
const DIAGNOSTIC_COUNT: usize = 256;
const DIAGNOSTIC_WIDTH: usize = 128;
const CLASSIFICATION_ITERATIONS: usize = 256;
const CLASSIFICATION_DIAGNOSTIC_COUNT: usize = 4_096;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_diagnostics(count: usize, width: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("plugin-{index:04}-{}", "x".repeat(width)))
        .collect()
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn legacy_unloaded_label(diagnostics: &[String], visits: &mut usize) -> &'static str {
    if diagnostics.iter().any(|diagnostic| {
        *visits += 1;
        diagnostic.contains("library is missing")
    }) {
        return "missing library";
    }
    if diagnostics.iter().any(|diagnostic| {
        *visits += 1;
        diagnostic.contains("failed to load")
    }) {
        return "load failed";
    }
    "manifest only"
}

#[test]
fn editor06_native_status_preserves_diagnostic_priority() {
    let diagnostics = vec![
        "plugin failed to load during discovery".to_string(),
        "plugin library is missing".to_string(),
    ];

    assert_eq!(
        native_load_state_label(false, false, &diagnostics),
        "missing library"
    );
    assert_eq!(
        native_load_state_label(true, true, &["plugin entry failed: panic".to_string()]),
        "entry failed"
    );
    assert_eq!(
        native_load_state_label(true, false, &["warning".to_string()]),
        "loaded without descriptor"
    );
    assert_eq!(native_load_state_label(true, true, &[]), "loaded");
    assert_eq!(native_load_state_label(false, false, &[]), "manifest only");
}

#[test]
fn editor06_native_status_reuses_materialized_diagnostics() {
    let classifier = include_str!("../native_load_state.rs");
    let caller = include_str!("../native.rs");

    assert!(!classifier.contains("diagnostics_for_plugin(plugin_id)"));
    assert_eq!(
        caller
            .matches("diagnostics_for_plugin(&package.id)")
            .count(),
        1
    );
    assert!(caller.contains("&package_diagnostics"));
}

#[test]
fn editor06_native_status_classifies_unloaded_diagnostics_in_one_pass() {
    let source = include_str!("../native_load_state.rs");

    assert!(source.contains("for diagnostic in diagnostics"));
    assert!(!source.contains("diagnostics\n        .iter()\n        .any"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_native_status_reused_diagnostics_bench() {
    let diagnostics = fixture_diagnostics(DIAGNOSTIC_COUNT, DIAGNOSTIC_WIDTH);
    let legacy = measure_samples(|| {
        for _ in 0..REUSE_ITERATIONS {
            let status_diagnostics = black_box(diagnostics.clone());
            let classifier_diagnostics = black_box(diagnostics.clone());
            black_box(native_load_state_label(
                false,
                false,
                &classifier_diagnostics,
            ));
            black_box((status_diagnostics, classifier_diagnostics));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..REUSE_ITERATIONS {
            let status_diagnostics = black_box(diagnostics.clone());
            black_box(native_load_state_label(false, false, &status_diagnostics));
            black_box(status_diagnostics);
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "EDITOR06_REUSED_NATIVE_STATUS_DIAGNOSTICS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} diagnostics={} diagnostic_width={} diagnostic_vec_materializations=2->1 diagnostic_string_clones={}->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        REUSE_ITERATIONS,
        DIAGNOSTIC_COUNT,
        DIAGNOSTIC_WIDTH,
        DIAGNOSTIC_COUNT * 2,
        DIAGNOSTIC_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 75,
        "optimized p95 should be at most 75% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_native_status_single_pass_classification_bench() {
    let mut diagnostics = fixture_diagnostics(CLASSIFICATION_DIAGNOSTIC_COUNT, 32);
    diagnostics
        .last_mut()
        .expect("classification fixture")
        .push_str(" failed to load");

    let mut legacy_visits = 0usize;
    let legacy = measure_samples(|| {
        for _ in 0..CLASSIFICATION_ITERATIONS {
            black_box(legacy_unloaded_label(&diagnostics, &mut legacy_visits));
        }
    });
    let mut optimized_visits = 0usize;
    let optimized = measure_samples(|| {
        for _ in 0..CLASSIFICATION_ITERATIONS {
            black_box(native_load_state_label(
                false,
                false,
                diagnostics
                    .iter()
                    .inspect(|_| optimized_visits += 1)
                    .map(|diagnostic| *diagnostic),
            ));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    let expected_legacy_visits =
        SAMPLE_COUNT * CLASSIFICATION_ITERATIONS * CLASSIFICATION_DIAGNOSTIC_COUNT * 2;
    let expected_optimized_visits =
        SAMPLE_COUNT * CLASSIFICATION_ITERATIONS * CLASSIFICATION_DIAGNOSTIC_COUNT;

    println!(
        "EDITOR06_SINGLE_PASS_NATIVE_LOAD_STATE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} diagnostics={} diagnostic_visits_per_iteration={}->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        CLASSIFICATION_ITERATIONS,
        CLASSIFICATION_DIAGNOSTIC_COUNT,
        CLASSIFICATION_DIAGNOSTIC_COUNT * 2,
        CLASSIFICATION_DIAGNOSTIC_COUNT,
    );
    assert_eq!(legacy_visits, expected_legacy_visits);
    assert_eq!(optimized_visits, expected_optimized_visits);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 70,
        "optimized p95 should be at most 70% of legacy p95"
    );
}
