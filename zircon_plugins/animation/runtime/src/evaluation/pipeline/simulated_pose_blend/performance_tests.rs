use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::math::{Quat, Transform, Vec3};

use super::blend_simulated_transform;

const ITERATIONS: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn optimization_batch_20260830cw_zero_weight_preserves_current_transform() {
    let current = current_transform();
    let blended = blend_simulated_transform(current, simulated_transform(), 0.0);

    assert_eq!(blended.translation, current.translation);
    assert_eq!(blended.scale, current.scale);
    assert_rotation_equivalent(blended.rotation, current.rotation.normalize());
}

#[test]
fn optimization_batch_20260830cw_full_weight_uses_simulated_transform() {
    let current = current_transform();
    let simulated = simulated_transform();
    let blended = blend_simulated_transform(current, simulated, 1.0);

    assert_eq!(blended.translation, simulated.translation);
    assert_eq!(blended.scale, simulated.scale);
    assert_rotation_equivalent(blended.rotation, simulated.rotation.normalize());
    assert!(blended.rotation.dot(current.rotation) >= 0.0);
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830cw_zero_weight_skips_simulated_pose_interpolation() {
    benchmark_endpoint("zero", 0.0);
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830cw_full_weight_skips_simulated_pose_interpolation() {
    benchmark_endpoint("full", 1.0);
}

fn benchmark_endpoint(endpoint: &str, weight: f32) {
    let current = current_transform();
    let simulated = simulated_transform();
    let baseline = measure_p95(|| {
        for _ in 0..ITERATIONS {
            black_box(baseline_blend(
                black_box(current),
                black_box(simulated),
                black_box(weight),
            ));
        }
    });
    let optimized = measure_p95(|| {
        for _ in 0..ITERATIONS {
            black_box(blend_simulated_transform(
                black_box(current),
                black_box(simulated),
                black_box(weight),
            ));
        }
    });
    let improvement = baseline.saturating_sub(optimized).saturating_mul(100) / baseline.max(1);

    println!(
        "PERF_RESULT task=runtime170_simulated_pose_{endpoint}_endpoint iterations={ITERATIONS} sample_pairs={SAMPLE_PAIRS} baseline_p95_ns={baseline} optimized_p95_ns={optimized} improvement_percent={improvement} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}"
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "simulated-pose {endpoint} endpoint must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn baseline_blend(current: Transform, simulated: Transform, weight: f32) -> Transform {
    Transform {
        translation: current.translation.lerp(simulated.translation, weight),
        rotation: current
            .rotation
            .slerp(simulated.rotation.normalize(), weight)
            .normalize(),
        scale: current.scale.lerp(simulated.scale, weight),
    }
}

fn current_transform() -> Transform {
    Transform {
        translation: Vec3::new(0.1, -0.2, 0.3),
        rotation: Quat::from_xyzw(0.04, 0.18, -0.11, 0.97).normalize(),
        scale: Vec3::new(1.0, 0.9, 1.1),
    }
}

fn simulated_transform() -> Transform {
    Transform {
        translation: Vec3::new(1.4, 2.1, -0.7),
        rotation: Quat::from_xyzw(-0.24, 0.38, 0.11, 0.88),
        scale: Vec3::new(0.8, 1.2, 1.0),
    }
}

fn assert_rotation_equivalent(actual: Quat, expected: Quat) {
    assert!(actual.dot(expected).abs() >= 1.0 - 1.0e-6);
}

fn measure_p95(mut run: impl FnMut()) -> u128 {
    let mut samples = Vec::with_capacity(SAMPLE_PAIRS);
    for _ in 0..SAMPLE_PAIRS {
        let started = Instant::now();
        run();
        samples.push(started.elapsed().as_nanos());
    }
    samples.sort_unstable();
    samples[SAMPLE_PAIRS * 95 / 100]
}
