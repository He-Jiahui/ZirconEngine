use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::{UiChildMount, UiNodeDefinition};

use super::{
    grid_slot_target_capacity, grid_slot_targets, UiAssetPaletteHoverContext,
    UiAssetPaletteNativeSlotTarget,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const ROWS_PER_BUILD: usize = 16;
const COLUMNS_PER_BUILD: usize = 16;
const TARGETS_PER_BUILD: usize = ROWS_PER_BUILD * COLUMNS_PER_BUILD;

#[test]
fn optimization_batch_20260826et_editor135_capacity_preserves_dynamic_grid_targets() {
    let node = UiNodeDefinition {
        children: vec![UiChildMount {
            slot: BTreeMap::from([
                ("row".to_string(), Value::Integer(15)),
                ("column".to_string(), Value::Integer(15)),
            ]),
            ..UiChildMount::default()
        }],
        ..UiNodeDefinition::default()
    };

    let targets = grid_slot_targets(
        &node,
        UiAssetPaletteHoverContext::new(10.0, 20.0, 320.0, 160.0, 170.0, 100.0),
    );

    assert_eq!(targets.len(), TARGETS_PER_BUILD);
    assert!(targets.capacity() >= TARGETS_PER_BUILD);
    assert_eq!(targets[0].label, "R0 C0");
    assert_eq!(targets[255].label, "R15 C15");
    assert_eq!(targets[255].slot.get("row"), Some(&Value::Integer(15)));
    assert_eq!(targets[255].slot.get("column"), Some(&Value::Integer(15)));
    assert_eq!(grid_slot_target_capacity(0, 0), 0);
    assert_eq!(
        grid_slot_target_capacity(ROWS_PER_BUILD, COLUMNS_PER_BUILD),
        TARGETS_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826et_editor135_grid_targets_reserve_axis_product() {
    let source = include_str!("../grid_slots.rs");
    assert!(source.contains("Vec::with_capacity(grid_slot_target_capacity(rows, columns))"));
    assert!(source.contains("rows.saturating_mul(columns)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826et_editor135_grid_slot_target_capacity_bench() {
    let target = UiAssetPaletteNativeSlotTarget {
        label: String::new(),
        detail: String::new(),
        slot: BTreeMap::new(),
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 10.0,
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
        "EDITOR135_GRID_SLOT_TARGET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={ROWS_PER_BUILD} \
columns_per_build={COLUMNS_PER_BUILD} targets_per_build={TARGETS_PER_BUILD} \
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
            Vec::with_capacity(TARGETS_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..TARGETS_PER_BUILD {
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
