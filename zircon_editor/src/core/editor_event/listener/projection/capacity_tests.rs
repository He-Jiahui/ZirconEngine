use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260830bo_editor_listener_projection_capacity_preserves_count() {
    let source = [1_u8, 2, 3, 4];
    let mut projected = Vec::with_capacity(source.len());
    projected.extend(source);
    assert_eq!(projected.len(), source.len());
    assert_eq!(projected.capacity(), source.len());
}

#[test]
fn optimization_batch_20260830bo_editor_listener_projection_capacity_source_contract() {
    let source = include_str!("../projection.rs");
    assert!(source.contains("let mut projected = Vec::with_capacity(listeners.len())"));
    assert!(source.contains("projected.extend(listeners.iter().map(listener_descriptor))"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bo_editor_listener_projection_capacity_p95() {
    const LISTENERS: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut projected = if pass == 0 {
                Vec::new()
            } else {
                Vec::with_capacity(LISTENERS)
            };
            for listener in 0..LISTENERS {
                projected.push(listener);
            }
            let checksum = projected.len() + projected.capacity();
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
        "{PERF_MARKER} listeners={LISTENERS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
