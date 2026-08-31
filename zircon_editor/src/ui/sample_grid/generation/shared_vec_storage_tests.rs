use std::hint::black_box;
use std::time::Instant;

use super::*;

const ITEM_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826ho_editor207_preserves_vec_allocation_and_shared_clones() {
    let values = (0..128).collect::<Vec<_>>();
    let allocation = values.as_ptr();
    let shared = shared_vec(values);
    let cloned = Arc::clone(&shared);

    assert_eq!(shared.as_slice(), &(0..128).collect::<Vec<_>>());
    assert_eq!(shared.as_ptr(), allocation);
    assert!(Arc::ptr_eq(&shared, &cloned));
}

#[test]
fn optimization_batch_20260826ho_editor207_uses_shared_vec_storage_for_grid_arrays() {
    let source = include_str!("../generation.rs");

    assert_eq!(source.matches("Arc<Vec<").count(), 4);
    assert_eq!(source.matches("shared_vec(").count(), 3);
    assert!(source.contains("Arc::new(values)"));
    assert!(!source.contains("Arc<[SampleGridTick]>"));
    assert!(!source.contains("Arc<[SampleGridPoint]>"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826ho_editor207_shared_vec_grid_storage_release_benchmark() {
    let source = (0..ITEM_COUNT).collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_shared_slice(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(shared_vec(black_box(&source).to_vec()));
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
        "EDITOR207_SHARED_VEC_GRID_STORAGE_BENCH_V1 \
         item_count={ITEM_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
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

fn legacy_shared_slice(source: &[usize]) -> Arc<[usize]> {
    source.to_vec().into()
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
