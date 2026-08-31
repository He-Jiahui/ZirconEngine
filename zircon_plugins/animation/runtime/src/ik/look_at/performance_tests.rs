use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::math::{Quat, Real, Vec3};

use super::LookAtJob;

const SAMPLE_COUNT: usize = 21;
const ITERATIONS: usize = 65_536;

#[test]
fn optimization_batch_20260830ct_zero_weight_look_at_preserves_normalized_rotation() {
    let current = Quat::from_xyzw(0.1, -0.2, 0.3, 0.9);
    let solved = LookAtJob::new(Vec3::Y, Vec3::X)
        .with_clamp_degrees(35.0)
        .with_weight(0.0)
        .solve_rotation(current)
        .expect("valid zero-weight look-at job");

    assert_eq!(solved, current.normalize());
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830ct_zero_weight_look_at_skips_full_solver() {
    let job = LookAtJob::new(Vec3::new(0.13, 0.98, -0.17), Vec3::X)
        .with_clamp_degrees(47.0)
        .with_weight(0.0);
    let current = Quat::from_rotation_y(0.37) * Quat::from_rotation_z(-0.22);

    let baseline = measure_p95(|| {
        let job = black_box(job);
        black_box(baseline_solve_rotation(job, black_box(current)));
    });
    let optimized = measure_p95(|| {
        let job = black_box(job);
        black_box(job.solve_rotation(black_box(current)).unwrap());
    });

    assert!(
        optimized * 100 <= baseline * 80,
        "zero-weight look-at p95 did not improve by 20%: baseline={baseline}ns optimized={optimized}ns"
    );
}

fn baseline_solve_rotation(job: LookAtJob, current: Quat) -> Quat {
    let current_axis = current * job.local_axis.normalize();
    let target = job.target_direction.normalize();
    let full_delta = Quat::from_rotation_arc(current_axis, target);
    let angle = full_delta.angle_between(Quat::IDENTITY);
    let limit = job
        .clamp_degrees
        .to_radians()
        .clamp(0.0, std::f32::consts::PI);
    let fraction = if angle <= Real::EPSILON {
        0.0
    } else {
        (limit / angle).min(1.0)
    };
    let delta = Quat::IDENTITY.slerp(full_delta, fraction * job.weight);
    (delta * current).normalize()
}

fn measure_p95(mut run: impl FnMut()) -> u128 {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            run();
        }
        samples.push(started.elapsed().as_nanos());
    }
    samples.sort_unstable();
    samples[SAMPLE_COUNT * 95 / 100]
}
