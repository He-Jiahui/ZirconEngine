use std::{
    collections::BTreeSet,
    hint::black_box,
    time::{Duration, Instant},
};

use super::append_node_and_cluster_cull_page_requests;

const BENCHMARK_REQUEST_COUNT: usize = 16_384;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn indexed_page_requests_preserve_first_seen_order_across_waves() {
    let mut page_request_ids = vec![3, 1];
    let mut seen_page_request_ids = page_request_ids.iter().copied().collect();

    append_node_and_cluster_cull_page_requests(
        &mut page_request_ids,
        &mut seen_page_request_ids,
        &[1, 2, 2, 4],
        4,
    );
    append_node_and_cluster_cull_page_requests(
        &mut page_request_ids,
        &mut seen_page_request_ids,
        &[5, 3],
        4,
    );

    assert_eq!(page_request_ids, vec![3, 1, 2, 4]);
    assert_eq!(seen_page_request_ids, BTreeSet::from([1, 2, 3, 4]));
}

#[test]
fn indexed_page_request_membership_performance_contract() {
    let requested_page_ids = (0..BENCHMARK_REQUEST_COUNT as u32).collect::<Vec<_>>();
    let legacy = || {
        let mut output = Vec::new();
        for page_id in &requested_page_ids {
            if !output.contains(page_id) {
                output.push(*page_id);
            }
        }
        black_box(output);
    };
    let optimized = || {
        let mut output = Vec::new();
        let mut seen = BTreeSet::new();
        append_node_and_cluster_cull_page_requests(
            &mut output,
            &mut seen,
            &requested_page_ids,
            BENCHMARK_REQUEST_COUNT as u32,
        );
        black_box(output);
    };

    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();
    let legacy_membership_comparisons = BENCHMARK_REQUEST_COUNT * (BENCHMARK_REQUEST_COUNT - 1) / 2;

    println!(
        "PERF_RESULT plugins17_indexed_cull_page_requests requests={BENCHMARK_REQUEST_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_membership_comparisons_per_sample={legacy_membership_comparisons} optimized_set_insertions_per_sample={BENCHMARK_REQUEST_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
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
