use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::math::Vec2;

use super::super::PrecisionShape;

const PERF_MARKER: &str = "EDITOR313_RING_THRESHOLD_EXIT_BENCH_V1";

fn ring() -> PrecisionShape {
    PrecisionShape::Ring {
        segments: (0..256)
            .map(|index| {
                let x = index as f32;
                (Vec2::new(x, 0.0), Vec2::new(x + 1.0, 0.0))
            })
            .collect(),
        radius_px: 1.0,
        thickness_px: 4.0,
        threshold_px: 2.0,
        depth: 0.0,
    }
}

#[test]
fn optimization_batch_20260830bo_editor_ring_threshold_exit_preserves_nonzero_score() {
    let shape = ring();
    assert_eq!(shape.score(Vec2::new(0.0, 4.0)), Some(0.0));
    assert_eq!(shape.score(Vec2::new(0.0, 5.0)), Some(1.0));
    assert_eq!(shape.score(Vec2::new(0.0, 6.0)), Some(2.0));
    assert_eq!(shape.score(Vec2::new(0.0, 7.0)), None);
}

#[test]
fn optimization_batch_20260830bo_editor_ring_threshold_exit_source_contract() {
    let source = include_str!("../precision_shape_score.rs");
    assert!(source.contains("best <= *thickness_px + *threshold_px"));
    assert!(source.contains("return Some(best - *thickness_px)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bo_editor_ring_threshold_exit_p95() {
    const SEGMENTS: usize = 256;
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let shape = black_box(ring());
    let point = black_box(Vec2::new(0.0, 6.0));
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let score = if pass == 0 {
                    let mut best = f32::MAX;
                    if let PrecisionShape::Ring { segments, .. } = &shape {
                        for (start, end) in segments {
                            let distance = crate::scene::viewport::projection::distance_to_segment(
                                point, *start, *end,
                            );
                            best = best.min(distance);
                        }
                    }
                    let score = best - 4.0;
                    (score <= 2.0).then_some(score.max(0.0)).unwrap_or(f32::MAX)
                } else {
                    shape.score(point).unwrap_or(f32::MAX)
                };
                checksum += usize::from(score < f32::MAX);
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
        "{PERF_MARKER} segments={SEGMENTS} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
