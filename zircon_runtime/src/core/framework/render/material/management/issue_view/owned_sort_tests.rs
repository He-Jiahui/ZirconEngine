use std::hint::black_box;
use std::time::Instant;

use super::*;

const RECORD_COUNT: usize = 8 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hn_runtime260_sorts_owned_storage_in_place() {
    let values = vec![
        "charlie".to_string(),
        "alpha".to_string(),
        "bravo".to_string(),
    ];
    let allocation = values.as_ptr();
    let sorted = sort_owned_values(values, |records| records.sort());

    assert_eq!(
        sorted,
        vec![
            "alpha".to_string(),
            "bravo".to_string(),
            "charlie".to_string()
        ]
    );
    assert_eq!(sorted.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260826hn_runtime260_owned_constructors_skip_second_clone() {
    let source = include_str!("../issue_view.rs");

    assert_eq!(source.matches(".into_sorted(sort_order)").count(), 3);
    assert!(source.contains("sort_owned_values(self.records"));
    assert!(source.contains("fn into_sorted(mut self"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hn_runtime260_owned_issue_view_sort_release_benchmark() {
    let source = (0..RECORD_COUNT)
        .map(|index| format!("issue-{index:08}-{}", "x".repeat(64)))
        .collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_sorted_view(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(owned_sorted_view(black_box(&source)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME260_OWNED_ISSUE_VIEW_SORT_BENCH_V1 \
         record_count={RECORD_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_sorted_view(source: &[String]) -> Vec<String> {
    let owned = source.to_vec();
    let mut records = owned.clone();
    records.sort();
    records
}

fn owned_sorted_view(source: &[String]) -> Vec<String> {
    let owned = source.to_vec();
    sort_owned_values(owned, |records| records.sort())
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
