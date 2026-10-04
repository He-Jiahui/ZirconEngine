use std::hint::black_box;
use std::time::Instant;

use super::NodeResourceCollector;

const SAMPLE_PAIRS: usize = 17;
const NODE_COUNT: usize = 4_096;

#[test]
fn runtime843_node_resource_collector_reserves_capacity() {
    let collector = NodeResourceCollector::with_capacity(3);
    assert!(collector.uris.capacity() >= 3);
    assert!(collector.seen.is_empty());
}

#[test]
#[ignore = "managed Windows release performance gate"]
fn runtime843_node_resource_registration_capacity_release_benchmark() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_growths = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_growths = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if pair % 2 == 0 {
            (measure_report(false), measure_report(true))
        } else {
            let optimized = measure_report(true);
            (measure_report(false), optimized)
        };
        legacy_samples.push(legacy.0);
        optimized_samples.push(optimized.0);
        legacy_growths.push(legacy.1);
        optimized_growths.push(optimized.1);
    }

    let legacy_p95 = percentile(&legacy_samples);
    let optimized_p95 = percentile(&optimized_samples);
    let legacy_growth_events = percentile_usize(&legacy_growths);
    let optimized_growth_events = percentile_usize(&optimized_growths);
    println!(
        "RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} node_count={NODE_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_ns={} optimized_ns={}",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert_eq!(optimized_growth_events, 0);
    assert!(optimized_p95 <= legacy_p95.saturating_mul(80) / 100);
}

fn measure_report(reserve: bool) -> (u128, usize) {
    let started = Instant::now();
    let mut growth_events = 0;
    let mut nodes_without_resources = Vec::new();
    if reserve {
        nodes_without_resources.reserve(NODE_COUNT);
    }
    for node_id in 0..NODE_COUNT {
        let capacity = nodes_without_resources.capacity();
        nodes_without_resources.push(black_box(node_id));
        growth_events += usize::from(nodes_without_resources.capacity() != capacity);
    }
    black_box(nodes_without_resources);
    (started.elapsed().as_nanos().max(1), growth_events)
}

fn percentile(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn percentile_usize(samples: &[usize]) -> usize {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
