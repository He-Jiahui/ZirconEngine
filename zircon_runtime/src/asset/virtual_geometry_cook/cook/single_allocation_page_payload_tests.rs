use std::hint::black_box;
use std::time::Instant;

use super::page_payload_byte_capacity;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 40_000;
const PAYLOAD_ITEM_COUNT: usize = 64;

#[test]
fn optimization_batch_20260829aj_runtime310_page_payload_capacity_is_exact() {
    assert_eq!(page_payload_byte_capacity(1), 44);
    assert_eq!(page_payload_byte_capacity(64), 1_052);
}

#[test]
fn optimization_batch_20260829aj_runtime310_page_payload_reserves_final_bytes() {
    let source = include_str!("../cook.rs");
    let builder = source
        .split("fn append_page_payload")
        .nth(1)
        .expect("page payload builder")
        .split("fn page_payload_byte_capacity")
        .next()
        .expect("page payload builder body");

    assert!(builder.contains("Vec::with_capacity(page_payload_byte_capacity(payload_item_count))"));
    assert!(!builder.contains("let mut payload = Vec::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aj_runtime310_single_allocation_page_payload_bench() {
    assert_eq!(
        optimized_payload(PAYLOAD_ITEM_COUNT),
        legacy_payload(PAYLOAD_ITEM_COUNT)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME310_SINGLE_ALLOCATION_VIRTUAL_GEOMETRY_PAGE_PAYLOAD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} payload_items_per_build={PAYLOAD_ITEM_COUNT} \
legacy_capacity_growth_enabled=1 optimized_capacity_growth_enabled=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_payload(item_count: usize) -> Vec<u8> {
    build_payload(Vec::new(), item_count)
}

fn optimized_payload(item_count: usize) -> Vec<u8> {
    build_payload(
        Vec::with_capacity(page_payload_byte_capacity(item_count)),
        item_count,
    )
}

fn build_payload(mut payload: Vec<u8>, item_count: usize) -> Vec<u8> {
    for word in 0..7 + item_count * 4 {
        payload.extend((word as u32).to_le_bytes());
    }
    payload
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let payload = if optimized {
            optimized_payload(black_box(PAYLOAD_ITEM_COUNT))
        } else {
            legacy_payload(black_box(PAYLOAD_ITEM_COUNT))
        };
        checksum = checksum.wrapping_add(black_box(payload).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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
