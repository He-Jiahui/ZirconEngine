use std::hint::black_box;
use std::time::Instant;

const NOTIFICATION_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_it_runtime630_reserves_shutdown_notification_batch() {
    let source = include_str!("../../lane.rs");
    let shutdown_body = source
        .split("pub fn shutdown(&self)")
        .nth(1)
        .expect("bounded keyed I/O shutdown remains present")
        .split("impl LaneInner")
        .next()
        .expect("shutdown implementation remains bounded");

    assert!(shutdown_body
        .contains("Vec::with_capacity(state.suspended.len().saturating_add(state.queue.len()))"));
    assert!(!shutdown_body.contains("let mut notifications = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_it_runtime630_preallocated_shutdown_notification_benchmark() {
    for _ in 0..4 {
        black_box(measure_notification_batch(false));
        black_box(measure_notification_batch(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_notification_batch(false));
            preallocated_samples.push(measure_notification_batch(true));
        } else {
            preallocated_samples.push(measure_notification_batch(true));
            unreserved_samples.push(measure_notification_batch(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME630_PREALLOCATED_KEYED_IO_SHUTDOWN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} notification_count={NOTIFICATION_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_notification_batch(preallocated: bool) -> u128 {
    let mut notifications = if preallocated {
        Vec::with_capacity(NOTIFICATION_COUNT)
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for notification in black_box(0..NOTIFICATION_COUNT) {
        notifications.push(notification);
    }
    black_box(notifications);
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
