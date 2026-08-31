use std::hint::black_box;
use std::time::Instant;

use super::*;

const RECORD_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hl_runtime258_preserves_cloned_query_iteration() {
    let source = vec!["alpha".to_string(), "beta".to_string()];
    let cloned = iter_cloned_values(&source).collect::<Vec<_>>();

    assert_eq!(cloned, source);
    assert_eq!(source.len(), 2);
}

#[test]
fn optimization_batch_20260826hl_runtime258_streams_overview_query_records() {
    let source = include_str!("../overview.rs");
    let start = source
        .find("pub fn query(")
        .expect("overview query function");
    let end = source[start..]
        .find("\n    }")
        .map(|offset| start + offset)
        .expect("overview query boundary");
    let body = &source[start..end];

    assert!(body.contains("iter_cloned_values(&self.records)"));
    assert!(!body.contains("self.records.clone()"));
    assert!(source.contains("source.iter().cloned()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hl_runtime258_streaming_overview_query_release_benchmark() {
    let source = (0..RECORD_COUNT)
        .map(|value| value as u64)
        .collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_filtered_query(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(streaming_filtered_query(black_box(&source)));
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
        "RUNTIME258_STREAMING_OVERVIEW_QUERY_BENCH_V1 \
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

fn legacy_filtered_query(source: &[u64]) -> Vec<u64> {
    source
        .to_vec()
        .into_iter()
        .filter(|value| value % 2 == 0)
        .collect()
}

fn streaming_filtered_query(source: &[u64]) -> Vec<u64> {
    iter_cloned_values(source)
        .filter(|value| value % 2 == 0)
        .collect()
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
