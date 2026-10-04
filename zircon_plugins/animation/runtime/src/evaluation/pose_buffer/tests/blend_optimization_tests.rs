use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::math::{Quat, Transform, Vec3};

use super::PoseBuffer;

#[test]
fn optimization_batch_20260830ch_zero_effective_weight_preserves_pose() {
    let mut destination = PoseBuffer::new(1);
    let expected = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(0.5),
        scale: Vec3::splat(1.5),
    };
    destination.set_transform(0, expected).unwrap();
    let mut source = PoseBuffer::new(1);
    source
        .set_transform(0, Transform::from_translation(Vec3::splat(9.0)))
        .unwrap();
    source.set_weight(0, 0.0).unwrap();

    destination.blend_override(&source, 1.0).unwrap();
    assert_eq!(destination.transform(0), Some(expected));
    destination.accumulate_additive(&source, 1.0).unwrap();
    assert_eq!(destination.transform(0), Some(expected));
}

#[test]
fn optimization_batch_20260830ch_zero_effective_weight_static_contract() {
    let production = include_str!("../blend.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert_eq!(production.matches("if effective_weight == 0.0").count(), 2);
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830ch_zero_effective_weight_p95() {
    const JOINTS: usize = 256;
    const ITERATIONS: usize = 20_000;
    const SAMPLES: usize = 17;
    let source = vec![[0.2_f32, 0.3, 0.4, 0.5]; JOINTS];
    let weights = vec![1.0_f32; JOINTS];
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let mut destination = vec![[0.0_f32, 0.0, 0.0, 1.0]; JOINTS];
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                zero_weight_blend_model(&mut destination, &source, &weights, false);
            }
            black_box(destination);
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let mut destination = vec![[0.0_f32, 0.0, 0.0, 1.0]; JOINTS];
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                zero_weight_blend_model(&mut destination, &source, &weights, true);
            }
            black_box(destination);
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_ZERO_EFFECTIVE_WEIGHT_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(70),
        "expected zero-weight early exit to reduce P95 by at least 30%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn zero_weight_blend_model(
    destination: &mut [[f32; 4]],
    source: &[[f32; 4]],
    weights: &[f32],
    skip_zero: bool,
) {
    for index in 0..destination.len() {
        let weight = weights[index] * 0.0;
        if skip_zero && weight <= f32::EPSILON {
            continue;
        }
        for component in 0..4 {
            destination[index][component] +=
                (source[index][component] - destination[index][component]) * weight;
        }
        black_box(destination[index]);
    }
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
