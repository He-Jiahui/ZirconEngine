use std::hint::black_box;
use std::time::Instant;

use crate::virtual_geometry::VirtualGeometryRuntimeState;

use super::resident_evictable_pages;

const BENCH_PAGE_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn resident_evictable_projection_preserves_order_duplicates_and_filtering() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.insert_resident_page_slot(20, 2);
    state.insert_resident_page_slot(40, 4);

    assert_eq!(
        resident_evictable_pages(&state, &[40, 30, 20, 40]),
        vec![40, 20, 40]
    );
}

#[test]
#[ignore = "release-only resident evictable-page projection benchmark"]
fn resident_evictable_exact_preallocation_release_benchmark_evidence() {
    let (state, candidates) = benchmark_inputs();
    assert_eq!(
        resident_evictable_pages(&state, &candidates),
        legacy_resident_evictable_pages(&state, &candidates)
    );

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy(&state, &candidates),
        || measure_optimized(&state, &candidates),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=resident_evictable_exact_preallocation \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
candidate_page_count={BENCH_PAGE_COUNT} resident_page_count={BENCH_PAGE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_capacity_lower_bound=0 optimized_capacity_lower_bound={BENCH_PAGE_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(19),
        "exact resident evictable-page preallocation must reduce P95 by at least 5%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_resident_evictable_pages(
    state: &VirtualGeometryRuntimeState,
    candidates: &[u32],
) -> Vec<u32> {
    candidates
        .iter()
        .copied()
        .filter(|page_id| state.has_resident_page(*page_id))
        .collect()
}

fn benchmark_inputs() -> (VirtualGeometryRuntimeState, Vec<u32>) {
    let mut state = VirtualGeometryRuntimeState::default();
    for page_id in 0..BENCH_PAGE_COUNT as u32 {
        state.insert_resident_page_slot(page_id, page_id);
    }
    let candidates = (0..BENCH_PAGE_COUNT as u32).rev().collect();
    (state, candidates)
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

fn measure_legacy(state: &VirtualGeometryRuntimeState, candidates: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_resident_evictable_pages(
            black_box(state),
            black_box(candidates),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(state: &VirtualGeometryRuntimeState, candidates: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(resident_evictable_pages(
            black_box(state),
            black_box(candidates),
        ));
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
