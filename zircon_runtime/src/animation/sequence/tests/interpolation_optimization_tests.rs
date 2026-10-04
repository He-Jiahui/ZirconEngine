use std::hint::black_box;
use std::time::Instant;

use super::*;

const LEFT: [Real; 4] = [1.0, 2.0, 3.0, 4.0];
const LEFT_TANGENT: [Real; 4] = [0.2, 0.3, 0.4, 0.5];
const RIGHT: [Real; 4] = [5.0, 6.0, 7.0, 8.0];
const RIGHT_TANGENT: [Real; 4] = [0.6, 0.7, 0.8, 0.9];

#[test]
fn optimization_batch_20260831fb_runtime567_shared_basis_preserves_vec4_samples() {
    for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
        assert_eq!(
            hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t),
            legacy_hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
        );
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260831fb_runtime567_shared_basis_vec4_p95() {
    const SAMPLE_PAIRS: usize = 13;
    const ITERATIONS: u64 = 10_000_000;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, ITERATIONS));
            optimized.push(measure(true, ITERATIONS));
        } else {
            optimized.push(measure(true, ITERATIONS));
            legacy.push(measure(false, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME567_HERMITE_BASIS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85));
}

fn legacy_hermite_array<const N: usize>(
    left: &[Real; N],
    left_tangent: [Real; N],
    right: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    t: Real,
) -> [Real; N] {
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        let t2 = t * t;
        let t3 = t2 * t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        result[index] = h00 * left[index]
            + h10 * left_tangent[index] * duration
            + h01 * right[index]
            + h11 * right_tangent[index] * duration;
        index += 1;
    }
    result
}

fn measure(optimized: bool, iterations: u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0.0;
    for index in 0..iterations {
        let t = black_box(((index & 1023) as Real + 0.5) / 1024.0);
        let values = if optimized {
            hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
        } else {
            legacy_hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
        };
        checksum += values[(index as usize) & 3];
    }
    black_box(checksum);
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
