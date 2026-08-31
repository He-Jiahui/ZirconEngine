use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const VALIDATIONS_PER_SAMPLE: usize = 100_000;
const SLOTS: [ToolkitAreaSlot; 4] = [
    ToolkitAreaSlot::Center,
    ToolkitAreaSlot::Left,
    ToolkitAreaSlot::Right,
    ToolkitAreaSlot::Bottom,
];

fn area(slot: ToolkitAreaSlot) -> ToolkitArea {
    let tab = format!("toolkit.{slot:?}").to_lowercase();
    ToolkitArea::new(slot, [tab.clone()], tab).expect("valid toolkit area")
}

#[test]
fn optimization_batch_20260826ck_editor_toolkit_slot_bitset_preserves_order_and_duplicate_error() {
    let layout = ToolkitLayout::new("toolkit.layout", SLOTS.map(area)).expect("valid layout");
    assert_eq!(
        layout
            .areas()
            .iter()
            .map(ToolkitArea::slot)
            .collect::<Vec<_>>(),
        SLOTS
    );

    let error = ToolkitLayout::new(
        "toolkit.duplicate",
        [
            area(ToolkitAreaSlot::Left),
            area(ToolkitAreaSlot::Right),
            area(ToolkitAreaSlot::Left),
        ],
    )
    .expect_err("duplicate slot must be rejected");
    assert_eq!(
        error,
        ToolkitLayoutError::DuplicateAreaSlot {
            slot: ToolkitAreaSlot::Left,
        }
    );
}

#[test]
fn optimization_batch_20260826ck_editor_toolkit_layout_uses_allocation_free_slot_bitset() {
    let source = include_str!("../layout.rs");
    let constructor = source
        .split("pub fn new")
        .nth(1)
        .and_then(|body| body.split("pub fn single_tab").next())
        .expect("ToolkitLayout constructor");

    assert!(constructor.contains("let mut occupied_slots = 0u8"));
    assert!(constructor.contains("area_slot_bit(area.slot())"));
    assert!(!constructor.contains("BTreeSet"));
}

fn legacy_slots_are_unique(slots: &[ToolkitAreaSlot]) -> bool {
    let mut occupied = BTreeSet::new();
    slots.iter().copied().all(|slot| occupied.insert(slot))
}

fn bitset_slots_are_unique(slots: &[ToolkitAreaSlot]) -> bool {
    let mut occupied = 0u8;
    slots.iter().copied().all(|slot| {
        let bit = area_slot_bit(slot);
        let unique = occupied & bit == 0;
        occupied |= bit;
        unique
    })
}

fn measure(validate: impl Fn(&[ToolkitAreaSlot]) -> bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..VALIDATIONS_PER_SAMPLE {
        checksum = checksum.wrapping_add(validate(black_box(&SLOTS)) as usize);
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

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
#[ignore = "release-only toolkit layout slot validation benchmark"]
fn optimization_batch_20260826ck_editor_toolkit_layout_slot_bitset_release_benchmark() {
    for _ in 0..4 {
        black_box(measure(legacy_slots_are_unique));
        black_box(measure(bitset_slots_are_unique));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_slots_are_unique));
            optimized_samples.push(measure(bitset_slots_are_unique));
        } else {
            optimized_samples.push(measure(bitset_slots_are_unique));
            legacy_samples.push(measure(legacy_slots_are_unique));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR50_TOOLKIT_LAYOUT_SLOT_BITSET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} slot_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_tree_instances_per_sample={VALIDATIONS_PER_SAMPLE} \
optimized_tree_instances_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        SLOTS.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "slot bitset must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
