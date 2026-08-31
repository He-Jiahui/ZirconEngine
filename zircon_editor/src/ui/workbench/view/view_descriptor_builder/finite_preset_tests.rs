use std::hint::black_box;
use std::time::Instant;

use super::{normalize_default_presets, DefaultWorkbenchPreset};

const MARKER: &str = "EDITOR189_VIEW_DEFAULT_PRESET_PRESENCE_TABLE_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 2_048;

#[test]
fn optimization_batch_20260826gw_editor189_default_presets_stay_unique_and_sorted() {
    let normalized = normalize_default_presets([
        DefaultWorkbenchPreset::Debug,
        DefaultWorkbenchPreset::Authoring,
        DefaultWorkbenchPreset::Focus,
        DefaultWorkbenchPreset::Debug,
        DefaultWorkbenchPreset::Review,
    ]);

    assert_eq!(
        normalized,
        [
            DefaultWorkbenchPreset::Authoring,
            DefaultWorkbenchPreset::Review,
            DefaultWorkbenchPreset::Focus,
            DefaultWorkbenchPreset::Debug,
        ]
    );
}

#[test]
fn optimization_batch_20260826gw_editor189_default_presets_use_fixed_presence_table() {
    let source = include_str!("../view_descriptor_builder.rs");
    let implementation = source
        .split("fn normalize_default_presets")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("default preset normalization implementation");
    assert!(implementation.contains("ORDERED_DEFAULT_PRESETS"));
    assert!(implementation.contains("let mut present = [false;"));
    assert!(!implementation.contains(".sort"));
    assert!(!implementation.contains(".dedup"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gw_editor189_view_default_preset_presence_table_bench() {
    let presets = (0..4_096)
        .map(|index| ORDERED_FIXTURE[index % ORDERED_FIXTURE.len()])
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&presets, legacy_normalize_default_presets));
            optimized_samples.push(measure(&presets, optimized_normalize_default_presets));
        } else {
            optimized_samples.push(measure(&presets, optimized_normalize_default_presets));
            legacy_samples.push(measure(&presets, legacy_normalize_default_presets));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "presence-table normalization must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

const ORDERED_FIXTURE: [DefaultWorkbenchPreset; 4] = [
    DefaultWorkbenchPreset::Authoring,
    DefaultWorkbenchPreset::Review,
    DefaultWorkbenchPreset::Focus,
    DefaultWorkbenchPreset::Debug,
];

fn legacy_normalize_default_presets(
    presets: &[DefaultWorkbenchPreset],
) -> Vec<DefaultWorkbenchPreset> {
    let mut presets = presets.to_vec();
    presets.sort();
    presets.dedup();
    presets
}

fn optimized_normalize_default_presets(
    presets: &[DefaultWorkbenchPreset],
) -> Vec<DefaultWorkbenchPreset> {
    normalize_default_presets(presets.iter().copied())
}

fn measure(
    presets: &[DefaultWorkbenchPreset],
    implementation: fn(&[DefaultWorkbenchPreset]) -> Vec<DefaultWorkbenchPreset>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let normalized = implementation(black_box(presets));
        checksum = checksum.wrapping_add(normalized.len());
        black_box(&normalized);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
