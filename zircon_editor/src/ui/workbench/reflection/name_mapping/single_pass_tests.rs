use std::hint::black_box;
use std::time::{Duration, Instant};

use super::menu_id;

const PERFORMANCE_MARKER: &str = "EDITOR85_MENU_ID_SINGLE_PASS_NORMALIZATION_BENCH_V1";

#[test]
fn optimization_batch_20260826cv_editor85_single_pass_menu_id_preserves_legacy_output() {
    for label in [
        "File",
        "Asset Build Export",
        "TWO  SPACES",
        "Cafe Menu",
        "\u{00c5}ngstrom Tools",
        "\u{6771}\u{4eac} WORKBENCH",
    ] {
        assert_eq!(menu_id(label), legacy_menu_id(label));
    }
}

#[test]
fn optimization_batch_20260826cv_editor85_single_pass_menu_id_uses_one_output_buffer() {
    let source = include_str!("../name_mapping.rs")
        .split_once("#[cfg(test)]")
        .expect("name mapping test boundary should exist")
        .0;

    assert!(source.contains("String::with_capacity(label.len())"));
    assert!(source.contains("menu_id.extend(label.chars().map"));
    assert!(!source.contains("to_ascii_lowercase().replace"));
}

#[test]
#[ignore = "release-only menu id normalization performance gate"]
fn optimization_batch_20260826cv_editor85_single_pass_menu_id_performance_evidence() {
    const LABEL_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR85_MENU_ID_SINGLE_PASS_NORMALIZATION_BENCH_V1"
    );
    let labels = (0..LABEL_COUNT)
        .map(|index| {
            format!(
                "ASSET WORKBENCH MATERIAL PIPELINE PROFILE {index:08} RUNTIME PREVIEW ACTION MENU"
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(normalize_batch(&labels, legacy_menu_id));
        black_box(normalize_batch(&labels, menu_id));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| normalize_batch(&labels, legacy_menu_id)));
            optimized_samples.push(measure(|| normalize_batch(&labels, menu_id)));
        } else {
            optimized_samples.push(measure(|| normalize_batch(&labels, menu_id)));
            legacy_samples.push(measure(|| normalize_batch(&labels, legacy_menu_id)));
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
        "single-pass menu id P95 {optimized_p95_ns}ns must be at most 70% of two-allocation legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_menu_id(label: &str) -> String {
    label.to_ascii_lowercase().replace(' ', "_")
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
