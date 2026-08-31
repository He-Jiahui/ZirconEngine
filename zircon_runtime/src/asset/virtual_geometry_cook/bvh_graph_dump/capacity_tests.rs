use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use super::{format_u32_list, u32_list_capacity};

const SAMPLE_PAIRS: usize = 21;
const LISTS_PER_SAMPLE: usize = 1_024;
const VALUES_PER_LIST: usize = 64;

#[test]
fn optimization_batch_20260826fo_runtime210_capacity_preserves_bvh_u32_list() {
    let values = vec![0, 7, 42, 1_024, u32::MAX];

    let formatted = format_u32_list(&values);

    assert_eq!(formatted, "[0,7,42,1024,4294967295]");
    assert!(formatted.capacity() >= u32_list_capacity(values.len()));
    assert_eq!(format_u32_list(&[]), "[]");
}

#[test]
fn optimization_batch_20260826fo_runtime210_bvh_list_reserves_u32_upper_bound() {
    let source = include_str!("../bvh_graph_dump.rs");
    assert!(source.contains("String::with_capacity(u32_list_capacity(values.len()))"));
    assert!(source.contains("const MAX_U32_DECIMAL_DIGITS: usize = 10;"));
    assert!(!source.contains("let mut formatted = String::from(\"[\");"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fo_runtime210_bvh_u32_list_capacity_bench() {
    let values = vec![u32::MAX; VALUES_PER_LIST];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&values, false));
            optimized_samples.push(measure(&values, true));
        } else {
            optimized_samples.push(measure(&values, true));
            legacy_samples.push(measure(&values, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME210_BVH_U32_LIST_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lists_per_sample={LISTS_PER_SAMPLE} values_per_list={VALUES_PER_LIST} \
legacy_reservations_per_list=0 optimized_reservations_per_list=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(values: &[u32], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LISTS_PER_SAMPLE {
        let formatted = if reserve {
            format_u32_list(black_box(values))
        } else {
            legacy_format_u32_list(black_box(values))
        };
        checksum ^= black_box(formatted.len() ^ formatted.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_format_u32_list(values: &[u32]) -> String {
    let mut formatted = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatted.push(',');
        }
        write!(formatted, "{value}").expect("writing to String cannot fail");
    }
    formatted.push(']');
    formatted
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
