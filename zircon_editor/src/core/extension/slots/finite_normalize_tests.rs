use std::hint::black_box;
use std::time::Instant;

use super::DefaultWorkbenchPreset;

const MARKER: &str = "EDITOR186_FINITE_WORKBENCH_PRESET_NORMALIZATION_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 512;

#[test]
fn optimization_batch_20260826gt_editor186_finite_normalization_preserves_canonical_order() {
    use DefaultWorkbenchPreset::{Authoring, Debug, Focus, Review};

    assert!(DefaultWorkbenchPreset::normalize([]).is_empty());
    assert_eq!(
        DefaultWorkbenchPreset::normalize([
            Debug, Review, Authoring, Focus, Debug, Authoring, Review,
        ]),
        [Authoring, Review, Focus, Debug]
    );
}

#[test]
fn optimization_batch_20260826gt_editor186_finite_normalization_uses_presence_table() {
    let source = include_str!("../slots.rs");
    let implementation = source
        .split("pub fn normalize")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("normalization implementation");
    assert!(implementation.contains("let mut present = [false; Self::COUNT]"));
    assert!(implementation.contains("Vec::with_capacity(present_count)"));
    assert!(!implementation.contains("collect::<Vec<_>>()"));
    assert!(!implementation.contains("sort_unstable"));
    assert!(!implementation.contains("dedup"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gt_editor186_finite_workbench_preset_normalization_bench() {
    use DefaultWorkbenchPreset::{Authoring, Debug, Focus, Review};

    let input = [Debug, Authoring, Focus, Review].repeat(1_024);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&input, legacy_normalize));
            optimized_samples.push(measure(&input, optimized_normalize));
        } else {
            optimized_samples.push(measure(&input, optimized_normalize));
            legacy_samples.push(measure(&input, legacy_normalize));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "finite-domain normalization must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_normalize(input: &[DefaultWorkbenchPreset]) -> Vec<DefaultWorkbenchPreset> {
    let mut presets = input.to_vec();
    presets.sort_unstable();
    presets.dedup();
    presets
}

fn optimized_normalize(input: &[DefaultWorkbenchPreset]) -> Vec<DefaultWorkbenchPreset> {
    DefaultWorkbenchPreset::normalize(input.iter().copied())
}

fn measure(
    input: &[DefaultWorkbenchPreset],
    implementation: fn(&[DefaultWorkbenchPreset]) -> Vec<DefaultWorkbenchPreset>,
) -> u64 {
    let started = Instant::now();
    let mut values = 0usize;
    for _ in 0..REPEATS {
        values += implementation(black_box(input)).len();
    }
    black_box(values);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
