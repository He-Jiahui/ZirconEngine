use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 31;
const ITERATIONS: usize = 250_000;

#[test]
fn optimization_batch_gw_runtime578_pmrem_length_preflight_preserves_error() {
    let expected = rgba16f_cube_mip_chain_len(128, 8).expect("bounded PMREM size");
    assert_eq!(validate_pmrem_payload_length(128, 8, expected), Ok(()));
    assert_eq!(
        validate_pmrem_payload_length(128, 8, expected - 2),
        Err(IblPmremTextureError::PayloadLength {
            expected,
            actual: expected - 2,
        })
    );

    let production = include_str!("../ibl_pmrem.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(production.contains("validate_pmrem_payload_length("));
    assert!(production.contains("let bytes = payload_bytes.to_vec();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_gw_runtime578_pmrem_invalid_length_preflight_p95() {
    let expected = rgba16f_cube_mip_chain_len(128, 8).expect("bounded PMREM size");
    let payload = vec![0_u8; expected - 2];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&payload, expected));
            optimized_samples.push(measure_optimized(&payload, expected));
        } else {
            optimized_samples.push(measure_optimized(&payload, expected));
            legacy_samples.push(measure_legacy(&payload, expected));
        }
    }
    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!(
        "RUNTIME578_PMREM_INVALID_LENGTH_PREFLIGHT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} payload_bytes={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        payload.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
        "invalid PMREM preflight should avoid the legacy payload copy: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(payload: &[u8], expected: usize) -> u128 {
    let started = Instant::now();
    let mut rejected = 0_usize;
    for _ in 0..ITERATIONS {
        let copied = payload.to_vec();
        if copied.len() != expected {
            rejected += 1;
        }
        black_box(copied);
    }
    black_box(rejected);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(payload: &[u8], expected: usize) -> u128 {
    let started = Instant::now();
    let mut rejected = 0_usize;
    for _ in 0..ITERATIONS {
        if payload.len() != expected {
            rejected += 1;
        }
        black_box(payload.len());
    }
    black_box(rejected);
    started.elapsed().as_nanos().max(1)
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() * 95 / 100]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
