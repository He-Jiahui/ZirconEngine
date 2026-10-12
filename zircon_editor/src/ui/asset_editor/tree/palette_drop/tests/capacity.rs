use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{
    overlay_slot_anchor, overlay_slot_targets, UiAssetPaletteHoverContext,
    UiAssetPaletteNativeSlotTarget, OVERLAY_SLOT_TARGET_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826es_editor134_capacity_preserves_nine_anchor_targets() {
    let targets = overlay_slot_targets(UiAssetPaletteHoverContext::new(
        10.0, 20.0, 300.0, 180.0, 160.0, 110.0,
    ));

    assert_eq!(targets.len(), OVERLAY_SLOT_TARGET_COUNT);
    assert!(targets.capacity() >= OVERLAY_SLOT_TARGET_COUNT);
    assert_eq!(targets[0].label, "Top Left");
    assert_eq!(overlay_slot_anchor(&targets[0].slot), Some((0.0, 0.0)));
    assert_eq!(targets[4].label, "Center");
    assert_eq!(overlay_slot_anchor(&targets[4].slot), Some((0.5, 0.5)));
    assert_eq!(targets[8].label, "Bottom Right");
    assert_eq!(overlay_slot_anchor(&targets[8].slot), Some((1.0, 1.0)));
}

#[test]
fn optimization_batch_20260826es_editor134_overlay_targets_reserve_fixed_grid_count() {
    let source = include_str!("../overlay_slots.rs");
    assert!(source.contains("const OVERLAY_SLOT_TARGET_COUNT: usize = 9;"));
    assert!(source.contains("Vec::with_capacity(OVERLAY_SLOT_TARGET_COUNT)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826es_editor134_overlay_slot_target_capacity_bench() {
    let target = UiAssetPaletteNativeSlotTarget {
        label: String::new(),
        detail: String::new(),
        slot: BTreeMap::new(),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 60.0,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&target, false));
            optimized_samples.push(measure(&target, true));
        } else {
            optimized_samples.push(measure(&target, true));
            legacy_samples.push(measure(&target, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR134_OVERLAY_SLOT_TARGET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} targets_per_build={OVERLAY_SLOT_TARGET_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(target: &UiAssetPaletteNativeSlotTarget, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut targets = if reserve {
            Vec::with_capacity(OVERLAY_SLOT_TARGET_COUNT)
        } else {
            Vec::new()
        };
        for _ in 0..OVERLAY_SLOT_TARGET_COUNT {
            targets.push(black_box(target.clone()));
        }
        checksum ^= black_box(targets.len() ^ targets.capacity());
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
