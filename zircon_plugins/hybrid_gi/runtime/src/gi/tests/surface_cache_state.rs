use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCH_ITEM_COUNT: usize = 4_096;
const BENCH_SAMPLE_PAIRS: usize = 21;

#[test]
fn surface_cache_indexed_slot_membership_preserves_duplicate_and_invalid_slot_behavior() {
    let item_ids = [10, 10, 20, 30, 40, 50];
    let previous_slots = BTreeMap::from([(10, 1), (20, 9), (30, 1), (40, 4)]);

    let legacy = legacy_assign_stable_slots(&item_ids, &previous_slots, 5);
    let indexed = assign_stable_slots(&item_ids, &previous_slots, 5, |item_id, slot_id| {
        (item_id, slot_id)
    });

    assert_eq!(indexed, legacy);
    assert_eq!(indexed, vec![(10, 1), (40, 4), (20, 0), (30, 2), (50, 3)]);
}

#[test]
#[ignore = "release performance gate; run through the managed Plugins19 validator"]
fn surface_cache_indexed_stable_slot_membership_release_benchmark() {
    let item_ids = (0..BENCH_ITEM_COUNT as u32).collect::<Vec<_>>();
    let previous_slots = item_ids
        .iter()
        .copied()
        .map(|item_id| (item_id, item_id))
        .collect::<BTreeMap<_, _>>();
    let expected = legacy_assign_stable_slots(
        black_box(&item_ids),
        black_box(&previous_slots),
        BENCH_ITEM_COUNT,
    );
    let indexed = assign_stable_slots(
        black_box(&item_ids),
        black_box(&previous_slots),
        BENCH_ITEM_COUNT,
        |item_id, slot_id| (item_id, slot_id),
    );
    assert_eq!(indexed, expected);

    let mut legacy_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    for pair_index in 0..BENCH_SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&item_ids, &previous_slots));
            optimized_samples.push(measure_indexed(&item_ids, &previous_slots));
        } else {
            optimized_samples.push(measure_indexed(&item_ids, &previous_slots));
            legacy_samples.push(measure_legacy(&item_ids, &previous_slots));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_linear_membership_comparisons =
        BENCH_ITEM_COUNT.saturating_mul(BENCH_ITEM_COUNT + 1) / 2;
    println!(
        "HYBRID_GI_STABLE_SLOT_MEMBERSHIP_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank item_count={} slot_capacity={} legacy_linear_membership_comparisons={} optimized_index_membership_lookups={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        BENCH_SAMPLE_PAIRS,
        BENCH_ITEM_COUNT,
        BENCH_ITEM_COUNT,
        legacy_linear_membership_comparisons,
        BENCH_ITEM_COUNT,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(4) <= legacy_p95,
        "indexed stable-slot membership P95 must be at most 25% of legacy: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn measure_legacy(item_ids: &[u32], previous_slots: &BTreeMap<u32, u32>) -> u128 {
    let started = Instant::now();
    black_box(legacy_assign_stable_slots(
        black_box(item_ids),
        black_box(previous_slots),
        BENCH_ITEM_COUNT,
    ));
    started.elapsed().as_nanos()
}

fn measure_indexed(item_ids: &[u32], previous_slots: &BTreeMap<u32, u32>) -> u128 {
    let started = Instant::now();
    black_box(assign_stable_slots(
        black_box(item_ids),
        black_box(previous_slots),
        BENCH_ITEM_COUNT,
        |item_id, slot_id| (item_id, slot_id),
    ));
    started.elapsed().as_nanos()
}

fn legacy_assign_stable_slots(
    item_ids: &[u32],
    previous_slots: &BTreeMap<u32, u32>,
    slot_capacity: usize,
) -> Vec<(u32, u32)> {
    let mut assigned_slots = Vec::with_capacity(item_ids.len());
    let mut used_slots = BTreeSet::new();

    for &item_id in item_ids {
        let Some(previous_slot_id) = previous_slots.get(&item_id).copied() else {
            continue;
        };
        if previous_slot_id as usize >= slot_capacity || !used_slots.insert(previous_slot_id) {
            continue;
        }
        assigned_slots.push((item_id, previous_slot_id));
    }

    let mut free_slots = (0..slot_capacity as u32)
        .filter(|slot_id| !used_slots.contains(slot_id))
        .collect::<Vec<_>>()
        .into_iter();
    for &item_id in item_ids {
        if assigned_slots
            .iter()
            .any(|(assigned_item_id, _)| *assigned_item_id == item_id)
        {
            continue;
        }
        let Some(slot_id) = free_slots.next() else {
            break;
        };
        assigned_slots.push((item_id, slot_id));
    }
    assigned_slots
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
