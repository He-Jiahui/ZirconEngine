use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ig_editor_panic_message_moves_owned_string() {
    let message = benchmark_panic_message(64 * 1024);
    let allocation = message.as_ptr();
    let payload: Box<dyn Any + Send> = Box::new(message);

    let recovered = panic_message(payload);

    assert_eq!(recovered.as_ptr(), allocation);
    assert!(recovered.starts_with("panic-message/"));
}

#[test]
fn optimization_batch_20260828ig_editor_panic_message_preserves_fallbacks() {
    assert_eq!(panic_message(Box::new("static panic")), "static panic");
    assert_eq!(panic_message(Box::new(41_u32)), "non-string panic payload");

    let source = include_str!("../pending_task.rs");
    let implementation = source
        .split("fn panic_message")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("panic message implementation");
    assert!(implementation.contains("payload.downcast::<String>()"));
    assert!(!implementation.contains("downcast_ref::<String>()"));
    assert!(!implementation.contains("message.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ig_editor_owned_panic_payload_string_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_panic_message(owned_panic_payload(64 * 1024)));
    black_box(panic_message(owned_panic_payload(64 * 1024)));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_payloads = benchmark_payloads(ITERATIONS, 64 * 1024);
        let optimized_payloads = benchmark_payloads(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_payloads(legacy_payloads, legacy_panic_message));
            optimized_samples.push(measure_payloads(optimized_payloads, panic_message));
        } else {
            optimized_samples.push(measure_payloads(optimized_payloads, panic_message));
            legacy_samples.push(measure_payloads(legacy_payloads, legacy_panic_message));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR225_OWNED_PANIC_PAYLOAD_STRING_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_panic_message(bytes: usize) -> String {
    "panic-message/".repeat(bytes / 14)
}

fn owned_panic_payload(bytes: usize) -> Box<dyn Any + Send> {
    Box::new(benchmark_panic_message(bytes))
}

fn benchmark_payloads(count: usize, bytes: usize) -> Vec<Box<dyn Any + Send>> {
    (0..count).map(|_| owned_panic_payload(bytes)).collect()
}

fn legacy_panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn measure_payloads(
    payloads: Vec<Box<dyn Any + Send>>,
    mut convert: impl FnMut(Box<dyn Any + Send>) -> String,
) -> u128 {
    let started = Instant::now();
    for payload in payloads {
        black_box(convert(black_box(payload)));
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
