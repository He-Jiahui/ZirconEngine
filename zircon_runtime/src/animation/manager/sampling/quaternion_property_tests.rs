use std::hint::black_box;
use std::time::Instant;

use super::{quaternion_sample_properties, sample_quaternion};
use crate::core::framework::animation::{AnimationChannelValueAsset, AnimationError};

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_properties(value: &[f32; 4]) -> (bool, bool) {
    (
        value.iter().all(|component| component.is_finite()),
        value
            .iter()
            .map(|component| component * component)
            .sum::<f32>()
            > f32::EPSILON,
    )
}

fn measure(value: &[f32; 4], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let properties = if optimized {
            quaternion_sample_properties(black_box(value))
        } else {
            legacy_properties(black_box(value))
        };
        evidence += properties.0 as usize + properties.1 as usize;
    }
    black_box(evidence);
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
fn optimization_batch_20260830cb_runtime355_quaternion_properties_preserve_results() {
    for value in [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
        [f32::NAN, 0.0, 0.0, 1.0],
    ] {
        assert_eq!(
            quaternion_sample_properties(&value),
            legacy_properties(&value)
        );
    }
    let finite = AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 1.0]);
    assert!(sample_quaternion(&finite).is_ok());
    let invalid = AnimationChannelValueAsset::Quaternion([f32::NAN, 0.0, 0.0, 1.0]);
    assert_eq!(
        sample_quaternion(&invalid),
        Err(AnimationError::NonFiniteSample {
            sample_kind: "quaternion"
        })
    );
}

#[test]
fn optimization_batch_20260830cb_runtime355_production_fuses_quaternion_scans() {
    let source = include_str!("../sampling.rs");
    let function = source
        .split_once("fn quaternion_sample_properties")
        .expect("property helper")
        .1;
    assert_eq!(function.matches("for component in value").count(), 1);
    assert!(
        !source.contains("value.iter().all(|c| c.is_finite()) && quaternion_array_is_normalizable")
    );
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260830cb_runtime355_quaternion_property_benchmark() {
    let value = [0.2, 0.4, 0.6, 0.7];
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&value, false));
            candidate.push(measure(&value, true));
        } else {
            candidate.push(measure(&value, true));
            baseline.push(measure(&value, false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME355_QUATERNION_PROPERTIES_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
