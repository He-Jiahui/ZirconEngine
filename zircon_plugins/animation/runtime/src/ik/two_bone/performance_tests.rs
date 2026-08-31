use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::math::{Real, Vec3};

use super::{bend_direction, TwoBoneIkJob, TwoBoneIkSolution};

const SAMPLE_COUNT: usize = 21;
const ITERATIONS: usize = 65_536;

#[test]
fn optimization_batch_20260830ct_zero_weight_two_bone_preserves_input_chain() {
    let root = Vec3::new(1.0, 2.0, 3.0);
    let mid = Vec3::new(2.0, 2.5, 3.0);
    let tip = Vec3::new(3.0, 2.0, 3.0);
    let solved = TwoBoneIkJob::new(Vec3::new(4.0, -1.0, 2.0))
        .with_pole(Vec3::Z)
        .with_weight(0.0)
        .solve_positions(root, mid, tip)
        .expect("valid zero-weight two-bone job");

    assert_eq!(solved, TwoBoneIkSolution { root, mid, tip });
}

#[test]
fn optimization_batch_20260830ct_zero_weight_two_bone_keeps_degenerate_error() {
    let error = TwoBoneIkJob::new(Vec3::X)
        .with_weight(0.0)
        .solve_positions(Vec3::ZERO, Vec3::ZERO, Vec3::X)
        .expect_err("zero weight must not bypass chain validation");

    assert_eq!(error, super::AnimationIkError::DegenerateChain);
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830ct_zero_weight_two_bone_skips_full_solver() {
    let job = TwoBoneIkJob::new(Vec3::new(1.25, 0.65, -0.15))
        .with_pole(Vec3::Z)
        .with_weight(0.0);
    let root = Vec3::new(0.1, -0.2, 0.3);
    let mid = Vec3::new(1.0, 0.15, 0.2);
    let tip = Vec3::new(1.85, -0.1, 0.4);

    let baseline = measure_p95(|| {
        let job = black_box(job);
        black_box(baseline_solve_positions(
            job,
            black_box(root),
            black_box(mid),
            black_box(tip),
        ));
    });
    let optimized = measure_p95(|| {
        let job = black_box(job);
        black_box(
            job.solve_positions(black_box(root), black_box(mid), black_box(tip))
                .unwrap(),
        );
    });

    assert!(
        optimized * 100 <= baseline * 80,
        "zero-weight two-bone p95 did not improve by 20%: baseline={baseline}ns optimized={optimized}ns"
    );
}

fn baseline_solve_positions(
    job: TwoBoneIkJob,
    root: Vec3,
    mid: Vec3,
    tip: Vec3,
) -> TwoBoneIkSolution {
    let upper = (mid - root).length();
    let lower = (tip - mid).length();
    let target_delta = job.target - root;
    let target_distance = target_delta.length();
    let direction = target_delta.try_normalize().unwrap_or(Vec3::X);
    let distance = target_distance.clamp((upper - lower).abs(), upper + lower);
    let pole = job.pole.unwrap_or(mid - root);
    let bend = bend_direction(direction, pole, mid - root);
    let along = ((upper * upper + distance * distance - lower * lower)
        / (2.0 * distance.max(Real::EPSILON)))
    .clamp(-upper, upper);
    let height = (upper * upper - along * along).max(0.0).sqrt();
    let solved_mid = root + direction * along + bend * height;
    let solved_tip = root + direction * distance;
    TwoBoneIkSolution {
        root,
        mid: mid.lerp(solved_mid, job.weight),
        tip: tip.lerp(solved_tip, job.weight),
    }
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
