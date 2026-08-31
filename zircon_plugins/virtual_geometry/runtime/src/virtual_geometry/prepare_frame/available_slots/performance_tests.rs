use std::hint::black_box;
use std::time::Instant;

use crate::virtual_geometry::VirtualGeometryRuntimeState;

use super::available_slots;

const BENCH_AVAILABLE_SLOT_COUNT: usize = 4_096;
const BENCH_FREE_SLOT_COUNT: usize = BENCH_AVAILABLE_SLOT_COUNT / 2;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn available_slot_projection_preserves_free_slot_order_before_future_slots() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.set_page_budget(6);
    state.insert_resident_page_slot(100, 0);
    state.insert_resident_page_slot(101, 1);
    state.insert_free_slot(9);
    state.insert_free_slot(3);
    state.insert_free_slot(7);
    state.advance_next_slot_past(11);

    assert_eq!(available_slots(&state), vec![3, 7, 9, 12]);
}

#[test]
#[ignore = "release-only available-slot projection benchmark"]
fn available_slot_exact_preallocation_release_benchmark_evidence() {
    let state = benchmark_state();
    assert_eq!(available_slots(&state), legacy_available_slots(&state));

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&state), || measure_optimized(&state));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=available_slot_exact_preallocation \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
available_slot_count={BENCH_AVAILABLE_SLOT_COUNT} free_slot_count={BENCH_FREE_SLOT_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_initial_capacity={BENCH_FREE_SLOT_COUNT} optimized_initial_capacity={BENCH_AVAILABLE_SLOT_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(19),
        "exact available-slot preallocation must reduce P95 by at least 5%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_available_slots(state: &VirtualGeometryRuntimeState) -> Vec<u32> {
    let available_slot_capacity = state
        .page_budget()
        .saturating_sub(state.resident_page_count());
    let mut available_slots = state
        .free_slot_ids()
        .take(available_slot_capacity)
        .collect::<Vec<_>>();
    let future_slot_count = available_slot_capacity.saturating_sub(available_slots.len());
    available_slots
        .extend((0..future_slot_count).map(|index| state.next_slot().saturating_add(index as u32)));
    available_slots
}

fn benchmark_state() -> VirtualGeometryRuntimeState {
    let mut state = VirtualGeometryRuntimeState::default();
    state.set_page_budget(BENCH_AVAILABLE_SLOT_COUNT);
    for slot in 0..BENCH_FREE_SLOT_COUNT as u32 {
        state.insert_free_slot(slot);
    }
    state.advance_next_slot_past(BENCH_AVAILABLE_SLOT_COUNT as u32 - 1);
    state
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy(state: &VirtualGeometryRuntimeState) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_available_slots(black_box(state)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(state: &VirtualGeometryRuntimeState) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(available_slots(black_box(state)));
    }
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
