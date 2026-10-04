use std::hint::black_box;
use std::time::Instant;

use super::sample_due;

#[test]
fn optimization_batch_20260831ez_runtime565_power_of_two_sampling_matches_modulo_semantics() {
    for interval in [1, 2, 4, 8, 64, 128] {
        for sample_index in 0..512 {
            assert_eq!(
                sample_due(sample_index, interval),
                sample_index % interval == 0,
                "interval={interval} sample_index={sample_index}"
            );
        }
    }
}

#[test]
fn optimization_batch_20260831ez_runtime565_non_power_of_two_sampling_keeps_modulo_semantics() {
    for interval in [3, 5, 7, 63, 65] {
        for sample_index in 0..512 {
            assert_eq!(
                sample_due(sample_index, interval),
                sample_index % interval == 0,
                "interval={interval} sample_index={sample_index}"
            );
        }
    }
    assert!(!sample_due(0, 0));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260831ez_runtime565_event_sampling_mask_p95() {
    const SAMPLE_PAIRS: usize = 13;
    const ITERATIONS: u64 = 20_000_000;
    const INTERVAL: u64 = 64;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, INTERVAL, ITERATIONS));
            optimized.push(measure(true, INTERVAL, ITERATIONS));
        } else {
            optimized.push(measure(true, INTERVAL, ITERATIONS));
            legacy.push(measure(false, INTERVAL, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME565_EVENT_SAMPLING_MASK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} interval={INTERVAL} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50));
}

fn measure(optimized: bool, interval: u64, iterations: u64) -> u128 {
    let started = Instant::now();
    let mut hits = 0_u64;
    let interval = black_box(interval);
    for sample_index in 0..iterations {
        hits += u64::from(if optimized {
            sample_due(sample_index, interval)
        } else {
            sample_index % interval == 0
        });
    }
    black_box(hits);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
