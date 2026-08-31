use std::hint::black_box;
use std::time::Instant;

use serde::Serialize;

use super::*;
use crate::scene::ecs::Event;

const TARGET_P95_PERCENT: u128 = 70;

#[derive(Serialize)]
struct BenchmarkMirrorEvent;

impl Event for BenchmarkMirrorEvent {}

#[test]
fn optimization_batch_20260828ij_runtime_unsubscribe_borrows_registration_event_id() {
    let registration = benchmark_registration(7);
    let allocation = registration.descriptor().event_id.as_ptr();

    let event_id = registration_event_id(&registration);

    assert_eq!(event_id.as_ptr(), allocation);
    assert_eq!(event_id, registration.descriptor().event_id);
}

#[test]
fn optimization_batch_20260828ij_runtime_unsubscribe_avoids_record_id_allocation() {
    let source = include_str!("../event_mirror.rs");
    let unsubscribe = source
        .split("pub fn unsubscribe_runtime_event_mirror")
        .nth(1)
        .and_then(|body| body.split("pub fn drain_runtime_event_mirror").next())
        .expect("runtime event mirror unsubscribe implementation");
    let borrowed_id = source
        .split("fn registration_event_id")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("registration event id helper");

    assert!(unsubscribe.contains("registration_event_id(&registration)"));
    assert!(!unsubscribe.contains("record.event_id().to_string()"));
    assert!(borrowed_id.contains("registration.descriptor().event_id.as_str()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ij_runtime_borrowed_event_mirror_unsubscribe_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 32 * 1024;

    let warm = benchmark_pairs(1);
    black_box(legacy_record_event_id(&warm[0].1));
    black_box(registration_event_id(&warm[0].0));

    let legacy_pairs = benchmark_pairs(ITERATIONS);
    let optimized_pairs = benchmark_pairs(ITERATIONS);
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy_ids(&legacy_pairs));
            optimized_samples.push(measure_borrowed_ids(&optimized_pairs));
        } else {
            optimized_samples.push(measure_borrowed_ids(&optimized_pairs));
            legacy_samples.push(measure_legacy_ids(&legacy_pairs));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME282_BORROWED_EVENT_MIRROR_UNSUBSCRIBE_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_registration(index: usize) -> RuntimeEventMirrorRegistration {
    RuntimeEventMirrorRegistration::typed::<BenchmarkMirrorEvent>(
        format!("runtime.event.{index:08}.{}", "x".repeat(96)),
        "benchmark-payload-v1",
    )
}

fn benchmark_pairs(
    count: usize,
) -> Vec<(
    RuntimeEventMirrorRegistration,
    RuntimeEventMirrorSubscriptionRecord,
)> {
    (0..count)
        .map(|index| {
            let registration = benchmark_registration(index);
            let record = registration.create_subscription_record();
            (registration, record)
        })
        .collect()
}

fn legacy_record_event_id(record: &RuntimeEventMirrorSubscriptionRecord) -> String {
    record.event_id().to_string()
}

fn measure_legacy_ids(
    pairs: &[(
        RuntimeEventMirrorRegistration,
        RuntimeEventMirrorSubscriptionRecord,
    )],
) -> u128 {
    let started = Instant::now();
    for (_, record) in pairs {
        black_box(legacy_record_event_id(black_box(record)));
    }
    started.elapsed().as_nanos()
}

fn measure_borrowed_ids(
    pairs: &[(
        RuntimeEventMirrorRegistration,
        RuntimeEventMirrorSubscriptionRecord,
    )],
) -> u128 {
    let started = Instant::now();
    for (registration, _) in pairs {
        let event_id = registration_event_id(black_box(registration));
        black_box((event_id.as_ptr(), event_id.len()));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
