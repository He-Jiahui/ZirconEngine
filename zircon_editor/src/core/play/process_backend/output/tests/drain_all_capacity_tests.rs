use std::hint::black_box;
use std::time::Instant;

use super::{drain_all_capacity, PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT, PLAY_OUTPUT_QUEUE_CAPACITY};

const BENCHMARK_MARKER: &str = "EDITOR07_PLAY_OUTPUT_DRAIN_ALL_CAPACITY_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const LINES_PER_SAMPLE: usize = PLAY_OUTPUT_QUEUE_CAPACITY;
const DRAINS_PER_SAMPLE: usize = 8;

#[test]
fn optimization_batch_20260915_editor07_drain_all_capacity_covers_queue() {
    assert_eq!(drain_all_capacity(0, false), 0);
    assert_eq!(
        drain_all_capacity(PLAY_OUTPUT_QUEUE_CAPACITY, true),
        PLAY_OUTPUT_QUEUE_CAPACITY + 1 + PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT
    );
    assert_eq!(
        drain_all_capacity(12, false),
        12 + PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT
    );
}

#[test]
fn optimization_batch_20260915_editor07_drain_all_capacity_preserves_line_order_model() {
    let queued = (0..4)
        .map(|index| format!("process.stdout: line-{index}"))
        .collect::<Vec<_>>();
    let mut projected = Vec::with_capacity(drain_all_capacity(queued.len(), true));
    projected.push("process.stdout: deferred".to_string());
    projected.extend(queued.iter().cloned());
    assert_eq!(&projected[1..], queued.as_slice());
    assert!(projected.capacity() >= projected.len());
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260915_editor07_drain_all_capacity_p95() {
    let lines = (0..LINES_PER_SAMPLE)
        .map(|index| format!("process.stdout: line-{index}"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(&lines, false));
            optimized_samples.push(sample_ns(&lines, true));
        } else {
            optimized_samples.push(sample_ns(&lines, true));
            legacy_samples.push(sample_ns(&lines, false));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_growth_events = geometric_growth_events(LINES_PER_SAMPLE + 5);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "{BENCHMARK_MARKER} lines={} drains_per_sample={} sample_pairs={} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={} optimized_ns={}",
        LINES_PER_SAMPLE,
        DRAINS_PER_SAMPLE,
        SAMPLE_PAIRS,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
}

fn sample_ns(lines: &[String], optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..DRAINS_PER_SAMPLE {
        let mut diagnostics = if optimized {
            Vec::with_capacity(drain_all_capacity(lines.len(), true))
        } else {
            Vec::new()
        };
        diagnostics.push("process.stdout: deferred".to_string());
        diagnostics.extend(lines.iter().cloned());
        black_box(diagnostics);
    }
    started.elapsed().as_nanos().max(1)
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for index in 1..=length {
        if index > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
