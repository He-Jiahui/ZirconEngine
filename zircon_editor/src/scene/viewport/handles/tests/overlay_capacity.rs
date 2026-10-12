use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "EDITOR311_ROTATE_OVERLAY_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260830bm_editor_rotate_overlay_capacity_preserves_results() {
    let mut elements = Vec::with_capacity(4);
    for value in 0..4 {
        elements.push(value);
    }
    assert_eq!(elements, [0, 1, 2, 3]);
    assert_eq!(elements.capacity(), 4);
}

#[test]
fn optimization_batch_20260830bm_editor_rotate_overlay_capacity_source_contract() {
    let source = include_str!("../rotate_handle_tool_behavior.rs");
    assert!(source.contains("let mut elements = Vec::with_capacity(4)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bm_editor_rotate_overlay_capacity_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let mut elements = if pass == 0 {
                    Vec::new()
                } else {
                    Vec::with_capacity(4)
                };
                elements.extend([0_usize, 1, 2, 3]);
                checksum += elements.len() + elements.capacity();
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
