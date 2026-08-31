use std::{
    collections::BTreeSet,
    hint::black_box,
    time::{Duration, Instant},
};

use super::{append_node_and_cluster_cull_requested_page_id, insert_store_cluster_key};

const BENCHMARK_ITEM_COUNT: usize = 16_384;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn child_decision_indexes_preserve_first_seen_budgeted_requests_and_store_keys() {
    let mut requested_page_ids = Vec::new();
    let mut requested_page_id_set = BTreeSet::new();
    for page_id in [3, 1, 3, 2, 4] {
        append_node_and_cluster_cull_requested_page_id(
            &mut requested_page_ids,
            &mut requested_page_id_set,
            page_id,
            3,
        );
    }
    assert_eq!(requested_page_ids, vec![3, 1, 2]);

    let mut store_cluster_keys = BTreeSet::new();
    assert!(insert_store_cluster_key(&mut store_cluster_keys, 7, 99, 11));
    assert!(!insert_store_cluster_key(
        &mut store_cluster_keys,
        7,
        99,
        11
    ));
    assert!(insert_store_cluster_key(&mut store_cluster_keys, 7, 99, 12));
}

#[test]
fn child_decision_requested_page_index_performance_contract() {
    let page_ids = (0..BENCHMARK_ITEM_COUNT as u32).collect::<Vec<_>>();
    let legacy = || {
        let mut requested = Vec::new();
        for page_id in &page_ids {
            if !requested.contains(page_id) {
                requested.push(*page_id);
            }
        }
        black_box(requested);
    };
    let optimized = || {
        let mut requested = Vec::new();
        let mut requested_set = BTreeSet::new();
        for page_id in &page_ids {
            append_node_and_cluster_cull_requested_page_id(
                &mut requested,
                &mut requested_set,
                *page_id,
                BENCHMARK_ITEM_COUNT as u32,
            );
        }
        black_box(requested);
    };

    assert_faster_and_report(
        "plugins17_indexed_child_page_requests",
        "request_ids",
        legacy,
        optimized,
    );
}

#[test]
fn child_decision_store_cluster_index_performance_contract() {
    let keys = (0..BENCHMARK_ITEM_COUNT as u32)
        .map(|index| (7, 99, index))
        .collect::<Vec<_>>();
    let legacy = || {
        let mut stored = Vec::new();
        for key in &keys {
            if !stored.contains(key) {
                stored.push(*key);
            }
        }
        black_box(stored);
    };
    let optimized = || {
        let mut stored = BTreeSet::new();
        for &(instance_index, entity, cluster_array_index) in &keys {
            black_box(insert_store_cluster_key(
                &mut stored,
                instance_index,
                entity,
                cluster_array_index,
            ));
        }
        black_box(stored);
    };

    assert_faster_and_report(
        "plugins17_indexed_store_cluster_records",
        "store_keys",
        legacy,
        optimized,
    );
}

fn assert_faster_and_report(
    name: &str,
    dimension_name: &str,
    legacy: impl Fn(),
    optimized: impl Fn(),
) {
    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();
    let legacy_membership_comparisons = BENCHMARK_ITEM_COUNT * (BENCHMARK_ITEM_COUNT - 1) / 2;

    println!(
        "PERF_RESULT {name} {dimension_name}={BENCHMARK_ITEM_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_membership_comparisons_per_sample={legacy_membership_comparisons} optimized_set_insertions_per_sample={BENCHMARK_ITEM_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "indexed membership must beat quadratic Vec scans: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
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
