use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{record_first_track_slot, TargetSlot};

const SLOT_COUNT: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 30;

#[test]
fn dense_first_track_slots_return_the_original_track_index() {
    let mut slots = vec![None; 4];

    assert_eq!(
        record_first_track_slot(&mut slots, TargetSlot::new(2), 7),
        None
    );
    assert_eq!(
        record_first_track_slot(&mut slots, TargetSlot::new(2), 11),
        Some(7)
    );
    assert_eq!(
        record_first_track_slot(&mut slots, TargetSlot::new(1), 3),
        None
    );
}

#[test]
#[ignore = "release-only performance gate"]
fn dense_first_track_slot_release_benchmark_evidence() {
    let slots = shuffled_slots();
    let (legacy_samples, optimized_samples) = paired_samples(
        || legacy_first_tracks(&slots),
        || optimized_first_tracks(&slots),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime170_dense_first_track_slots slots={SLOT_COUNT} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_tree_nodes={SLOT_COUNT} optimized_tree_nodes=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert_eq!(legacy_first_tracks(&slots).len(), SLOT_COUNT);
    assert_eq!(
        optimized_first_tracks(&slots).into_iter().flatten().count(),
        SLOT_COUNT
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "dense first-track slots must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn shuffled_slots() -> Vec<TargetSlot> {
    let mut state = 0x4d59_5df4_d0f3_3173_u64;
    let mut slots = (0..SLOT_COUNT)
        .map(|slot| TargetSlot::new(slot as u32))
        .collect::<Vec<_>>();
    for index in (1..slots.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        slots.swap(index, state as usize % (index + 1));
    }
    slots
}

fn legacy_first_tracks(slots: &[TargetSlot]) -> BTreeMap<TargetSlot, usize> {
    let mut first_tracks = BTreeMap::new();
    for (track_index, slot) in slots.iter().copied().enumerate() {
        first_tracks.insert(slot, track_index);
    }
    first_tracks
}

fn optimized_first_tracks(slots: &[TargetSlot]) -> Vec<Option<usize>> {
    let mut first_tracks = vec![None; SLOT_COUNT];
    for (track_index, slot) in slots.iter().copied().enumerate() {
        assert_eq!(
            record_first_track_slot(&mut first_tracks, slot, track_index),
            None
        );
    }
    first_tracks
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ordered.len().saturating_mul(percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
