use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use crate::virtual_geometry::VirtualGeometryRuntimeState;

const BENCHMARK_CANDIDATE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn preferred_eviction_skips_non_resident_candidates_without_changing_priority() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.insert_resident_page_slot(20, 2);
    state.insert_resident_page_slot(10, 1);

    assert_eq!(
        state.preferred_resident_evictable_page_for_target(100, &[99, 20, 10]),
        Some(10)
    );
    assert!(state.evict_preferred_page_for_target(100, &[99, 20, 10]));
    assert!(!state.has_resident_page(10));
    assert!(state.has_resident_page(20));
}

#[test]
fn linear_resident_eviction_selection_performance_contract() {
    let mut state = VirtualGeometryRuntimeState::default();
    for page_id in 0..BENCHMARK_CANDIDATE_COUNT as u32 {
        state.insert_resident_page_slot(page_id, page_id);
    }
    let candidates = (0..BENCHMARK_CANDIDATE_COUNT as u32)
        .rev()
        .collect::<Vec<_>>();
    let target_page_id = 100_000;
    let legacy = || {
        black_box(
            state
                .ordered_evictable_pages_for_target(target_page_id, &candidates)
                .into_iter()
                .find(|page_id| state.has_resident_page(*page_id)),
        );
    };
    let optimized = || {
        black_box(state.preferred_resident_evictable_page_for_target(target_page_id, &candidates));
    };

    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();

    println!(
        "PERF_RESULT plugins17_linear_resident_eviction_selection candidates={BENCHMARK_CANDIDATE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_full_sorts_per_sample=1 optimized_full_sorts_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "linear resident selection must beat full sorting: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
    );
}

fn paired_samples(legacy: impl Fn(), optimized: impl Fn()) -> (Vec<Duration>, Vec<Duration>) {
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&legacy));
            optimized_samples.push(measure(&optimized));
        } else {
            optimized_samples.push(measure(&optimized));
            legacy_samples.push(measure(&legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(run: impl Fn()) -> Duration {
    let started = Instant::now();
    run();
    started.elapsed()
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}
