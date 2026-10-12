use std::{
    collections::BTreeMap,
    hint::black_box,
    time::{Duration, Instant},
};

use super::VirtualGeometryRuntimeState;

const BENCHMARK_PAGE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn descendant_ids_preserve_sorted_cycle_guard_semantics() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.replace_page_parent_pages(BTreeMap::from([(1, 0), (2, 1), (0, 2), (4, 0)]));

    assert_eq!(state.page_descendant_ids(0), vec![0, 1, 2, 4]);
    assert!(state.page_descendant_ids(99).is_empty());
}

#[test]
fn ordered_descendant_set_performance_contract() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.replace_page_parent_pages(
        (1..BENCHMARK_PAGE_COUNT as u32)
            .map(|page_id| (page_id, page_id - 1))
            .collect(),
    );
    let legacy = || {
        black_box(legacy_page_descendant_ids(&state, 0));
    };
    let optimized = || {
        black_box(state.page_descendant_ids(0));
    };

    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();
    let descendant_count = BENCHMARK_PAGE_COUNT - 1;
    let legacy_membership_comparisons = descendant_count * (descendant_count - 1) / 2;

    println!(
        "PERF_RESULT plugins17_ordered_descendant_set pages={BENCHMARK_PAGE_COUNT} descendants={descendant_count} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_membership_comparisons_per_sample={legacy_membership_comparisons} optimized_ordered_set_insertions_per_sample={descendant_count} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "ordered-set traversal must beat quadratic Vec membership scans: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
    );
}

fn legacy_page_descendant_ids(state: &VirtualGeometryRuntimeState, page_id: u32) -> Vec<u32> {
    let mut stack = state
        .page_child_pages()
        .get(&page_id)
        .cloned()
        .unwrap_or_default();
    let mut descendants = Vec::new();

    while let Some(candidate_page_id) = stack.pop() {
        if descendants.contains(&candidate_page_id) {
            continue;
        }
        descendants.push(candidate_page_id);
        if let Some(child_page_ids) = state.page_child_pages().get(&candidate_page_id) {
            stack.extend(child_page_ids.iter().copied());
        }
    }

    descendants.sort_unstable();
    descendants
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
