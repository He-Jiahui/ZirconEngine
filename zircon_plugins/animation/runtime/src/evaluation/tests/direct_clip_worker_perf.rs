use std::hint::black_box;
use std::time::Instant;

use super::{direct_clip_shard_capacity, new_direct_clip_batches};

const SAMPLE_PAIRS: usize = 21;
const BENCH_ITEMS: usize = 65_536;
const BENCH_SHARDS: usize = 4;

#[test]
fn optimization_batch_20260830co_direct_clip_capacity_matches_round_robin_peak() {
    assert_eq!(direct_clip_shard_capacity(0, 0), 0);
    assert_eq!(direct_clip_shard_capacity(1, 1), 1);
    assert_eq!(direct_clip_shard_capacity(10, 4), 3);

    let mut batches = new_direct_clip_batches::<usize>(10, 4);
    for item in 0..10 {
        batches[item % 4].push(item);
    }
    assert_eq!(
        batches.iter().map(Vec::len).collect::<Vec<_>>(),
        [3, 3, 2, 2]
    );
    assert!(batches.iter().all(|batch| batch.capacity() >= 3));
}

#[test]
#[ignore = "release-only direct clip shard capacity benchmark"]
fn optimization_batch_20260830co_direct_clip_capacity_release_benchmark() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_growth_events = 0usize;
    let mut optimized_growth_events = 0usize;

    for pair in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if pair % 2 == 0 {
            (measure_legacy(), measure_optimized())
        } else {
            let optimized = measure_optimized();
            let legacy = measure_legacy();
            (legacy, optimized)
        };
        assert_eq!(legacy.2, optimized.2);
        legacy_samples.push(legacy.0);
        optimized_samples.push(optimized.0);
        legacy_growth_events = legacy_growth_events.saturating_add(legacy.1);
        optimized_growth_events = optimized_growth_events.saturating_add(optimized.1);
    }

    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME170_DIRECT_CLIP_SHARD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items={BENCH_ITEMS} shards={BENCH_SHARDS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn measure_legacy() -> (u128, usize, usize) {
    let started = Instant::now();
    let mut growth_events = 0usize;
    let mut batches = (0..BENCH_SHARDS)
        .map(|_| Vec::new())
        .collect::<Vec<Vec<usize>>>();
    for item in 0..BENCH_ITEMS {
        let batch = &mut batches[item % BENCH_SHARDS];
        growth_events += usize::from(batch.len() == batch.capacity());
        batch.push(item);
    }
    let checksum = batches.iter().flatten().copied().sum::<usize>();
    (
        started.elapsed().as_nanos(),
        growth_events,
        black_box(checksum),
    )
}

fn measure_optimized() -> (u128, usize, usize) {
    let started = Instant::now();
    let mut growth_events = 0usize;
    let mut batches = new_direct_clip_batches(BENCH_ITEMS, BENCH_SHARDS);
    for item in 0..BENCH_ITEMS {
        let batch = &mut batches[item % BENCH_SHARDS];
        growth_events += usize::from(batch.len() == batch.capacity());
        batch.push(item);
    }
    let checksum = batches.iter().flatten().copied().sum::<usize>();
    (
        started.elapsed().as_nanos(),
        growth_events,
        black_box(checksum),
    )
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
