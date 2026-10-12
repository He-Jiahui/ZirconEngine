use std::hint::black_box;
use std::time::{Duration, Instant};

use super::label_to_action_segment;

const PERFORMANCE_MARKER: &str = "EDITOR88_POPUP_ACTION_SEGMENT_IN_PLACE_TRIM_BENCH_V1";

#[test]
fn optimization_batch_20260826cy_editor88_popup_action_segment_preserves_legacy_output() {
    for label in [
        "Open Project",
        "--Open--Project--",
        "HTTP Server 2",
        "already_snake_case",
        "\u{00c5}ngstrom Tool",
        "___",
        "",
    ] {
        assert_eq!(
            label_to_action_segment(label),
            legacy_label_to_action_segment(label),
            "{label}"
        );
    }
}

#[test]
fn optimization_batch_20260826cy_editor88_popup_action_segment_trims_in_place() {
    let source = include_str!("../popup_primitives.rs")
        .split_once("#[cfg(test)]")
        .expect("popup primitives test boundary should exist")
        .0;
    let normalization = source
        .split_once("fn label_to_action_segment")
        .expect("action segment helper should exist")
        .1;

    assert!(normalization.contains("String::with_capacity(label.len())"));
    assert!(normalization.contains("output.pop()"));
    assert!(!normalization.contains("trim_matches('_').to_string()"));
}

#[test]
#[ignore = "release-only popup action segment performance gate"]
fn optimization_batch_20260826cy_editor88_popup_action_segment_performance_evidence() {
    const LABEL_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR88_POPUP_ACTION_SEGMENT_IN_PLACE_TRIM_BENCH_V1"
    );
    let labels = (0..LABEL_COUNT)
        .map(|index| format!("--CreateMaterialPreviewAction{index:08}SelectedAssetChanged--"))
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(normalize_batch(&labels, legacy_label_to_action_segment));
        black_box(normalize_batch(&labels, label_to_action_segment));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                normalize_batch(&labels, legacy_label_to_action_segment)
            }));
            optimized_samples.push(measure(|| {
                normalize_batch(&labels, label_to_action_segment)
            }));
        } else {
            optimized_samples.push(measure(|| {
                normalize_batch(&labels, label_to_action_segment)
            }));
            legacy_samples.push(measure(|| {
                normalize_batch(&labels, legacy_label_to_action_segment)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} labels={LABEL_COUNT} samples={SAMPLE_COUNT} legacy_allocations_per_label=2 optimized_allocations_per_label=1"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place popup segment P95 {optimized_p95_ns}ns must be at most 70% of copied-trim P95 {legacy_p95_ns}ns"
    );
}

fn legacy_label_to_action_segment(label: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
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
