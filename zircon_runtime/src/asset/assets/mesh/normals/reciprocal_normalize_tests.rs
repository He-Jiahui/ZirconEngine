use std::hint::black_box;
use std::time::Instant;

use super::super::normalize;

const SAMPLE_PAIRS: usize = 31;
const VECTOR_COUNT: usize = 262_144;

fn legacy_normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length > 0.0 {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn fixture_vectors() -> Vec<[f32; 3]> {
    (0..VECTOR_COUNT)
        .map(|index| {
            [
                index as f32 + 1.0,
                (index % 257) as f32 + 0.25,
                (index % 997) as f32 + 0.5,
            ]
        })
        .collect()
}

fn measure(vectors: &[[f32; 3]], optimized: bool) -> u128 {
    let started = Instant::now();
    let sum = vectors.iter().fold([0.0_f32; 3], |mut sum, vector| {
        let normalized = if optimized {
            normalize(*black_box(vector))
        } else {
            legacy_normalize(*black_box(vector))
        };
        sum[0] += normalized[0];
        sum[1] += normalized[1];
        sum[2] += normalized[2];
        sum
    });
    black_box(sum);
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

#[test]
fn optimization_batch_20260829az_runtime327_reciprocal_normalization_preserves_vectors() {
    for vector in [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 2.0, 3.0],
        [-4.0, 5.0, -6.0],
        [f32::MIN_POSITIVE, 0.0, 0.0],
    ] {
        let legacy = legacy_normalize(vector);
        let optimized = normalize(vector);
        for component in 0..3 {
            assert!((legacy[component] - optimized[component]).abs() <= f32::EPSILON);
        }
    }
}

#[test]
fn optimization_batch_20260829az_runtime327_normalization_reuses_one_inverse_length() {
    let source = include_str!("../normals.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("let inverse_length = length.recip();"));
    assert_eq!(production.matches("* inverse_length").count(), 3);
    assert!(!production.contains("vector[0] / length"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829az_runtime327_reciprocal_normal_vector_bench() {
    let vectors = fixture_vectors();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&vectors, false));
            optimized_samples.push(measure(&vectors, true));
        } else {
            optimized_samples.push(measure(&vectors, true));
            legacy_samples.push(measure(&vectors, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME327_RECIPROCAL_NORMAL_VECTOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
vectors={VECTOR_COUNT} legacy_divisions_per_vector=3 optimized_divisions_per_vector=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
