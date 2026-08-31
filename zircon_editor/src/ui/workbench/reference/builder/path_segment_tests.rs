use std::hint::black_box;
use std::time::{Duration, Instant};

use super::normalized_reference_path_segment;

const PERFORMANCE_MARKER: &str = "EDITOR86_REFERENCE_PATH_SINGLE_PASS_NORMALIZATION_BENCH_V1";

#[test]
fn optimization_batch_20260826cw_editor86_path_segment_preserves_legacy_output() {
    for label in [
        "Ready",
        "No Errors",
        "Speed 0.25",
        "Position X 128.4",
        "TWO  SPACES",
        "\u{00c5}ngstrom WORKBENCH",
    ] {
        assert_eq!(
            normalized_reference_path_segment(label),
            legacy_path_segment(label)
        );
    }
}

#[test]
fn optimization_batch_20260826cw_editor86_builder_routes_double_normalization_through_helper() {
    let builder = include_str!("mod.rs")
        .split_once("#[cfg(test)]")
        .expect("builder test boundary should exist")
        .0;
    let panels = include_str!("panels.rs");

    assert!(builder.contains("String::with_capacity(label.len())"));
    assert_eq!(
        builder
            .matches("normalized_reference_path_segment(label)")
            .count()
            + panels
                .matches("normalized_reference_path_segment(label)")
                .count(),
        5
    );
    assert!(!builder.contains("replace(' ', \"_\").to_ascii_lowercase()"));
    assert!(!panels.contains("replace(' ', \"_\").to_ascii_lowercase()"));
}

#[test]
#[ignore = "release-only reference path normalization performance gate"]
fn optimization_batch_20260826cw_editor86_path_segment_performance_evidence() {
    const LABEL_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR86_REFERENCE_PATH_SINGLE_PASS_NORMALIZATION_BENCH_V1"
    );
    let labels = (0..LABEL_COUNT)
        .map(|index| {
            format!(
                "REFERENCE WORKBENCH MATERIAL INSPECTOR POSITION X {index:08} RUNTIME PREVIEW ROW"
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(normalize_batch(&labels, legacy_path_segment));
        black_box(normalize_batch(&labels, normalized_reference_path_segment));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| normalize_batch(&labels, legacy_path_segment)));
            optimized_samples.push(measure(|| {
                normalize_batch(&labels, normalized_reference_path_segment)
            }));
        } else {
            optimized_samples.push(measure(|| {
                normalize_batch(&labels, normalized_reference_path_segment)
            }));
            legacy_samples.push(measure(|| normalize_batch(&labels, legacy_path_segment)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} labels={LABEL_COUNT} call_sites=5 samples={SAMPLE_COUNT} legacy_allocations_per_label=2 optimized_allocations_per_label=1"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-pass reference path P95 {optimized_p95_ns}ns must be at most 70% of two-allocation legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_path_segment(label: &str) -> String {
    label.replace(' ', "_").to_ascii_lowercase()
}

fn normalize_batch(labels: &[String], normalize: fn(&str) -> String) -> usize {
    labels
        .iter()
        .map(|label| black_box(normalize(black_box(label))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
