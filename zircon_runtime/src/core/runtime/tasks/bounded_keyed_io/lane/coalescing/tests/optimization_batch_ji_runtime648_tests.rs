use std::hint::black_box;
use std::time::Instant;

const ENTRY_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ji_runtime648_preallocates_matching_queue_partition() {
    let source = include_str!("../../coalescing.rs");
    let partition = source
        .split("fn take_matching_entries")
        .nth(1)
        .expect("matching queue partition remains present")
        .split("pub(super) fn coalesce_queued_generation")
        .next()
        .expect("matching queue partition remains bounded");

    assert!(partition.contains("Vec::with_capacity(pending.len())"));
    assert!(!partition.contains("let mut matching = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ji_runtime648_matching_partition_capacity_benchmark() {
    let values = (0..ENTRY_COUNT).collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_partition(&values, false));
        black_box(measure_partition(&values, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_partition(&values, false));
            reserved_samples.push(measure_partition(&values, true));
        } else {
            reserved_samples.push(measure_partition(&values, true));
            unreserved_samples.push(measure_partition(&values, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let reserved_p95 = percentile(&reserved_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(reserved_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME648_MATCHING_PARTITION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entry_count={ENTRY_COUNT} unreserved_ns={} reserved_ns={} unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&reserved_samples),
    );
    assert!(reserved_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_partition(values: &[usize], reserved: bool) -> u128 {
    let started = Instant::now();
    let mut matching = if reserved {
        Vec::with_capacity(values.len())
    } else {
        Vec::new()
    };
    for value in values {
        if value % 2 == 0 {
            matching.push(black_box(*value));
        }
    }
    black_box(matching);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
