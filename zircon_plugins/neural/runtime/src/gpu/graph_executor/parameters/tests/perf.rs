use std::hint::black_box;
use std::time::Instant;

use super::{elementwise_parameters, gemm_parameters};

const ITERATIONS_PER_SAMPLE: usize = 32_768;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn fixed_parameter_buffers_preserve_wire_layout_and_zero_padding() {
    let gemm = gemm_parameters(2, 3, 4, 1.5, 0.25);
    assert_eq!(gemm.len(), 32);
    assert_eq!(&gemm[0..4], &2_u32.to_le_bytes());
    assert_eq!(&gemm[4..8], &3_u32.to_le_bytes());
    assert_eq!(&gemm[8..12], &4_u32.to_le_bytes());
    assert_eq!(&gemm[12..16], &[0; 4]);
    assert_eq!(&gemm[16..20], &1.5_f32.to_le_bytes());
    assert_eq!(&gemm[20..24], &0.25_f32.to_le_bytes());
    assert_eq!(&gemm[24..], &[0; 8]);

    let elementwise = elementwise_parameters(513);
    assert_eq!(elementwise.len(), 16);
    assert_eq!(&elementwise[0..4], &513_u32.to_le_bytes());
    assert_eq!(&elementwise[4..], &[0; 12]);
}

#[test]
#[ignore = "release-only fixed Gemm parameter buffer benchmark"]
fn fixed_gemm_parameter_buffer_release_benchmark_evidence() {
    assert_eq!(
        legacy_gemm_parameters(2, 3, 4, 1.5, 0.25),
        gemm_parameters(2, 3, 4, 1.5, 0.25)
    );
    run_parameter_gate(
        "fixed_gemm_parameters",
        "legacy_initial_capacity=24 optimized_fixed_bytes=32 legacy_growth_events=1 optimized_growth_events=0",
        || legacy_gemm_parameters(black_box(2), 3, 4, 1.5, 0.25),
        || gemm_parameters(black_box(2), 3, 4, 1.5, 0.25),
    );
}

#[test]
#[ignore = "release-only fixed elementwise parameter buffer benchmark"]
fn fixed_elementwise_parameter_buffer_release_benchmark_evidence() {
    assert_eq!(
        legacy_elementwise_parameters(513),
        elementwise_parameters(513)
    );
    run_parameter_gate(
        "fixed_elementwise_parameters",
        "legacy_initial_capacity=4 optimized_fixed_bytes=16 legacy_growth_events=1 optimized_growth_events=0",
        || legacy_elementwise_parameters(black_box(513)),
        || elementwise_parameters(black_box(513)),
    );
}

fn run_parameter_gate(
    task: &str,
    allocation_evidence: &str,
    mut legacy: impl FnMut() -> Vec<u8>,
    mut optimized: impl FnMut() -> Vec<u8>,
) {
    for _ in 0..256 {
        black_box(legacy());
        black_box(optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task={task} sample_pairs={SAMPLE_PAIRS} \
iterations_per_sample={ITERATIONS_PER_SAMPLE} {allocation_evidence} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "fixed parameter buffer must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure(build: &mut impl FnMut() -> Vec<u8>) -> u128 {
    let started = Instant::now();
    for _ in 0..ITERATIONS_PER_SAMPLE {
        black_box(build());
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_gemm_parameters(m: u32, n: u32, k: u32, alpha: f32, beta: f32) -> Vec<u8> {
    let mut parameters = Vec::with_capacity(24);
    for value in [m, n, k, 0] {
        parameters.extend_from_slice(&value.to_le_bytes());
    }
    parameters.extend_from_slice(&alpha.to_le_bytes());
    parameters.extend_from_slice(&beta.to_le_bytes());
    parameters.resize(32, 0);
    parameters
}

fn legacy_elementwise_parameters(elements: u32) -> Vec<u8> {
    let mut parameters = elements.to_le_bytes().to_vec();
    parameters.resize(16, 0);
    parameters
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
