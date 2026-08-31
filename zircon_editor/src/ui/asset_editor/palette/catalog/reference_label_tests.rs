use std::hint::black_box;
use std::time::Instant;

use super::reference_palette_label;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 131_072;
const REFERENCE: &str = "res://widgets/production_hud.zui#InventoryEquipmentGrid";

#[test]
fn optimization_batch_20260826dp_editor105_reference_palette_label_preserves_reference_forms() {
    assert_eq!(
        reference_palette_label("res://widgets/hud.zui#Inventory"),
        "Reference / Inventory"
    );
    assert_eq!(
        reference_palette_label("res://widgets/hud.zui"),
        "Reference / res://widgets/hud.zui"
    );
    assert_eq!(reference_palette_label("asset#"), "Reference / ");
}

#[test]
fn optimization_batch_20260826dp_editor105_reference_palette_label_formats_borrowed_slice() {
    let source = include_str!("../catalog.rs");
    assert!(source.contains("label: reference_palette_label(reference)"));
    assert!(source.contains(".map_or(reference, |(_, component)| component)"));
    assert!(!source.contains(".map(|(_, component)| component.to_string())"));
    assert!(!source.contains(".unwrap_or_else(|| reference.clone())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dp_editor105_reference_palette_label_borrowed_format_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_label));
            optimized_samples.push(measure(reference_palette_label));
        } else {
            optimized_samples.push(measure(reference_palette_label));
            legacy_samples.push(measure(legacy_label));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR105_REFERENCE_PALETTE_LABEL_BORROWED_FORMAT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} legacy_allocations_per_label=2 \
optimized_allocations_per_label=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed reference palette label P95 {optimized_p95_ns}ns must be at most 70% of copied label formatting P95 {legacy_p95_ns}ns"
    );
}

fn legacy_label(reference: &str) -> String {
    let label = reference
        .split_once('#')
        .map(|(_, component)| component.to_string())
        .unwrap_or_else(|| reference.to_string());
    format!("Reference / {label}")
}

fn measure(render: fn(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(REFERENCE))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
