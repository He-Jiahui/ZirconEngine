use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::math::Vec2;

use super::super::frame_from_points;

const PERF_MARKER: &str = "EDITOR307_VEC2_FRAME_BOUNDS_BENCH_V1";

fn points(count: usize) -> Vec<Vec2> {
    (0..count)
        .map(|index| {
            let x = index as f32 - 100.0;
            Vec2::new(x, -x * 0.5)
        })
        .collect()
}

#[test]
fn optimization_batch_20260830bj_editor_vec2_bounds_preserves_frame() {
    let frame = frame_from_points([Vec2::new(-2.0, 3.0), Vec2::new(4.0, -1.0)], 2.0)
        .expect("points should produce a frame");
    assert_eq!(frame.x, -4.0);
    assert_eq!(frame.y, -3.0);
    assert_eq!(frame.width, 10.0);
    assert_eq!(frame.height, 8.0);
}

#[test]
fn optimization_batch_20260830bj_editor_vec2_bounds_source_contract() {
    let source = include_str!("../frame_from_points.rs");
    assert!(source.contains("let mut min = first"));
    assert!(source.contains("min = min.min(point)"));
    assert!(source.contains("max = max.max(point)"));
    assert!(!source.contains("let mut min_x = first.x"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bj_editor_vec2_bounds_p95() {
    const POINTS: usize = 256;
    const REPETITIONS: usize = 20_000;
    const SAMPLES: usize = 17;
    let values = black_box(points(POINTS));
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0.0_f32;
            for _ in 0..REPETITIONS {
                let bounds = if pass == 0 {
                    let mut points = values.iter().copied();
                    let first = points.next().expect("points");
                    let mut min_x = first.x;
                    let mut min_y = first.y;
                    let mut max_x = first.x;
                    let mut max_y = first.y;
                    for point in points {
                        min_x = min_x.min(point.x);
                        min_y = min_y.min(point.y);
                        max_x = max_x.max(point.x);
                        max_y = max_y.max(point.y);
                    }
                    (min_x, min_y, max_x, max_y)
                } else {
                    let frame = frame_from_points(values.iter().copied(), 0.0).expect("points");
                    (frame.x, frame.y, frame.right(), frame.bottom())
                };
                checksum += bounds.0 + bounds.1 + bounds.2 + bounds.3;
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} points={POINTS} repetitions={REPETITIONS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
