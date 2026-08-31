use std::{
    collections::BTreeMap,
    hint::black_box,
    time::{Duration, Instant},
};

use crate::virtual_geometry::{VirtualGeometryPageRequest, VirtualGeometryRuntimeState};

use super::assigned_slots;

const BENCHMARK_CANDIDATE_COUNT: usize = 2_048;
const BENCHMARK_TARGET_COUNT: usize = 4;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn preferred_evictable_page_matches_full_sort_for_mixed_priority_state() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.replace_page_parent_pages(BTreeMap::from([
        (11, 10),
        (12, 11),
        (21, 20),
        (22, 21),
        (31, 30),
    ]));
    state.extend_current_hot_resident_pages([12, 30]);
    state.insert_pending_page(22);
    state.ensure_current_requested_page_order(22, 0);
    state.insert_pending_page(31);
    state.ensure_current_requested_page_order(31, 1);

    let candidates = [31, 10, 22, 20, 12, 30, 11, 21];
    for target_page_id in [12, 22, 31, 99] {
        let fully_sorted = state.ordered_evictable_pages_for_target(target_page_id, &candidates);

        assert_eq!(
            state.preferred_evictable_page_for_target(target_page_id, &candidates),
            fully_sorted.first().copied(),
            "linear selection must preserve the complete eviction ordering"
        );
    }
}

#[test]
fn assigned_slots_consume_each_recycled_page_once() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.set_page_budget(3);
    state.insert_resident_page_slot(30, 3);
    state.insert_resident_page_slot(10, 1);
    state.insert_resident_page_slot(20, 2);
    state.replace_evictable_pages(vec![30, 10, 20]);
    let requests = vec![
        VirtualGeometryPageRequest::new(100, 64, 1),
        VirtualGeometryPageRequest::new(101, 64, 1),
        VirtualGeometryPageRequest::new(102, 64, 1),
    ];

    let plans = assigned_slots(&state, &requests);

    assert_eq!(plans[&100].recycled_page_id, Some(10));
    assert_eq!(plans[&100].slot, Some(1));
    assert_eq!(plans[&101].recycled_page_id, Some(20));
    assert_eq!(plans[&101].slot, Some(2));
    assert_eq!(plans[&102].recycled_page_id, Some(30));
    assert_eq!(plans[&102].slot, Some(3));
}

#[test]
fn preferred_evictable_page_linear_selection_performance_contract() {
    let state = VirtualGeometryRuntimeState::default();
    let candidates = (0..BENCHMARK_CANDIDATE_COUNT as u32)
        .rev()
        .collect::<Vec<_>>();
    let targets = (0..BENCHMARK_TARGET_COUNT as u32)
        .map(|index| 100_000 + index)
        .collect::<Vec<_>>();

    let legacy = || {
        for target_page_id in &targets {
            black_box(
                state
                    .ordered_evictable_pages_for_target(*target_page_id, &candidates)
                    .first()
                    .copied(),
            );
        }
    };
    let optimized = || {
        for target_page_id in &targets {
            black_box(state.preferred_evictable_page_for_target(*target_page_id, &candidates));
        }
    };

    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();

    println!(
        "PERF_RESULT plugins17_linear_preferred_evictable_page candidates={BENCHMARK_CANDIDATE_COUNT} targets_per_sample={BENCHMARK_TARGET_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_full_sorts_per_sample={BENCHMARK_TARGET_COUNT} optimized_full_sorts_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "linear minimum selection must beat fully sorting every candidate: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
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
