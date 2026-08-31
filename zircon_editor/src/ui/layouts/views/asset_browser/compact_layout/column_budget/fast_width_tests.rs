use std::hint::black_box;
use std::time::Instant;

use super::side_panels_preserve_content;

const PERF_MARKER: &str = "EDITOR308_COMPACT_SIDE_WIDTH_FAST_PATH_BENCH_V1";

fn baseline_side_panels_preserve_content(
    viewport_width: f32,
    side_widths: &[f32],
    gap: f32,
) -> bool {
    if viewport_width <= f32::EPSILON {
        return false;
    }
    let (side_width, visible_side_count) = side_widths
        .iter()
        .copied()
        .filter(|width| *width > f32::EPSILON)
        .fold((0.0_f32, 0_usize), |(width, count), side_width| {
            (width + side_width, count + 1)
        });
    let remaining_content_width =
        (viewport_width - side_width - gap * visible_side_count as f32).max(0.0);
    remaining_content_width >= viewport_width * 0.52
}

#[test]
fn optimization_batch_20260830bk_editor_compact_side_width_preserves_results() {
    let cases = [
        (0.0, &[][..]),
        (900.0, &[152.0][..]),
        (900.0, &[152.0, 204.0][..]),
        (900.0, &[152.0, 204.0, 48.0][..]),
        (300.0, &[0.0, 204.0][..]),
    ];
    for (viewport, widths) in cases {
        assert_eq!(
            side_panels_preserve_content(viewport, widths, 6.0),
            baseline_side_panels_preserve_content(viewport, widths, 6.0),
            "viewport={viewport} widths={widths:?}"
        );
    }
}

#[test]
fn optimization_batch_20260830bk_editor_compact_side_width_source_contract() {
    let source = include_str!("../column_budget.rs");
    assert!(source.contains("match side_widths"));
    assert!(source.contains("[first, second]"));
    assert!(source.contains("_ => side_widths"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bk_editor_compact_side_width_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let widths = black_box([152.0_f32, 204.0_f32]);
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let fits = if pass == 0 {
                    baseline_side_panels_preserve_content(1200.0, &widths, 6.0)
                } else {
                    side_panels_preserve_content(1200.0, &widths, 6.0)
                };
                checksum += usize::from(fits);
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
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
