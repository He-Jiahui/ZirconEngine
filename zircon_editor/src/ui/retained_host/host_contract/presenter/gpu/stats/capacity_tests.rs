use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::rhi::UiSurfacePresentStats;

use super::{append_present_stats, present_stat_counter_count};

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 8_192;
const FULL_PRESENT_STAT_COUNTER_COUNT: usize = 55;

#[test]
fn optimization_batch_20260826fy_editor166_present_stats_reserve_exact_counter_count() {
    let mut stats = UiSurfacePresentStats::default();
    assert_eq!(present_stat_counter_count(&stats), 52);

    stats.gpu_timestamp_supported = true;
    stats.gpu_time_us = Some(125);
    assert_eq!(
        present_stat_counter_count(&stats),
        FULL_PRESENT_STAT_COUNTER_COUNT
    );

    let mut counters = Vec::new();
    append_present_stats(&mut counters, &stats, true);
    assert_eq!(counters.len(), FULL_PRESENT_STAT_COUNTER_COUNT);
    assert!(counters.capacity() >= FULL_PRESENT_STAT_COUNTER_COUNT);
}

#[test]
fn optimization_batch_20260826fy_editor166_present_stats_reserve_precedes_recording() {
    let source = include_str!("../stats.rs");
    let reserve = source
        .find("counters.reserve(present_stat_counter_count(stats));")
        .expect("present stats reserve");
    let recorder = source
        .find("let mut record = |counter, value|")
        .expect("counter recorder");

    assert!(reserve < recorder);
    assert!(source.contains("const BASE_PRESENT_STAT_COUNTER_COUNT: usize = 52;"));
    assert!(source.contains("usize::from(stats.gpu_time_us.is_some())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fy_editor166_present_stats_capacity_bench() {
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
        "EDITOR166_PRESENT_STATS_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} counters_per_batch={FULL_PRESENT_STAT_COUNTER_COUNT} \
legacy_preallocated_batches=0 optimized_preallocated_batches={BATCHES_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..BATCHES_PER_SAMPLE {
        let mut counters = if reserve {
            Vec::with_capacity(FULL_PRESENT_STAT_COUNTER_COUNT)
        } else {
            Vec::new()
        };
        for counter in 0..FULL_PRESENT_STAT_COUNTER_COUNT {
            counters.push((black_box(counter), black_box((batch ^ counter) as f64)));
        }
        checksum ^= black_box(counters.len() ^ counters.capacity());
        black_box(&counters);
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
