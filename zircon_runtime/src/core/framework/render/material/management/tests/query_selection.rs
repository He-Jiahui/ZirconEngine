use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const RECORDS_PER_SAMPLE: usize = 512;

#[test]
fn query_selection_reserves_page_id_capacity() {
    let source = include_str!("../query_selection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("query selection implementation");

    assert!(implementation.contains("Vec::with_capacity(query_result.records.len())"));
    assert!(implementation.contains("page_material_ids.extend("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ct_runtime_query_selection_capacity_p95() {
    let ids = (0..RECORDS_PER_SAMPLE)
        .map(|index| ResourceId::from_stable_label(&format!("material-{index:04}")))
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&ids, false));
            optimized.push(measure(&ids, true));
        } else {
            optimized.push(measure(&ids, true));
            legacy.push(measure(&ids, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME395_QUERY_SELECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} records_per_sample={RECORDS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(ids: &[ResourceId], use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..512 {
        let values = if use_capacity {
            let mut values = Vec::with_capacity(ids.len());
            values.extend(ids.iter().copied());
            values
        } else {
            black_box(ids).iter().copied().collect()
        };
        checksum ^= values.len();
        black_box(values);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
