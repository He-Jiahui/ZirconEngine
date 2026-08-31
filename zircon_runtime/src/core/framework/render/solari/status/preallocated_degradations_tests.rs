use std::hint::black_box;
use std::time::Instant;

use super::{SolariCapabilityRequirement, SOLARI_MAX_DEGRADATION_COUNT};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 200_000;

#[test]
fn optimization_batch_20260829ak_runtime311_solari_capacity_covers_every_degradation() {
    assert_eq!(
        SOLARI_MAX_DEGRADATION_COUNT,
        SolariCapabilityRequirement::ALL.len() + 3
    );
}

#[test]
fn optimization_batch_20260829ak_runtime311_report_preallocates_the_bounded_vector() {
    let source = include_str!("../status.rs");
    let builder = source
        .split("pub fn from_inputs")
        .nth(1)
        .expect("Solari report builder")
        .split("pub fn enabled")
        .next()
        .expect("Solari report builder body");

    assert!(builder.contains("Vec::with_capacity(SOLARI_MAX_DEGRADATION_COUNT)"));
    assert!(!builder.contains("let mut degradations = Vec::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ak_runtime311_preallocated_solari_degradations_bench() {
    assert_eq!(optimized_degradations(), legacy_degradations());

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
        "RUNTIME311_PREALLOCATED_SOLARI_DEGRADATIONS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} degradations_per_build={SOLARI_MAX_DEGRADATION_COUNT} \
legacy_vector_allocations_per_build=3 optimized_vector_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_degradations() -> Vec<usize> {
    build_degradations(Vec::new())
}

fn optimized_degradations() -> Vec<usize> {
    build_degradations(Vec::with_capacity(SOLARI_MAX_DEGRADATION_COUNT))
}

fn build_degradations(mut degradations: Vec<usize>) -> Vec<usize> {
    for degradation in 0..SOLARI_MAX_DEGRADATION_COUNT {
        degradations.push(degradation);
    }
    degradations
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let degradations = if optimized {
            optimized_degradations()
        } else {
            legacy_degradations()
        };
        checksum = checksum.wrapping_add(black_box(degradations).len());
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
