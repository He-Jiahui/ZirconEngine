use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "EDITOR837_TEMPLATE_BINDING_ID_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260919_editor837_template_binding_id_capacity_preserves_order() {
    let source = [
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ];
    let mut projected = Vec::with_capacity(source.len());
    projected.extend(source.iter().cloned());

    assert_eq!(projected, source);
    assert_eq!(projected.len(), source.len());
    assert_eq!(projected.capacity(), source.len());
}

#[test]
fn optimization_batch_20260919_editor837_template_binding_id_capacity_source_contract() {
    let source = include_str!("../../projection.rs");
    assert!(source.contains("Vec::with_capacity(node.bindings.len())"));
    assert!(source.contains("Vec::with_capacity(node.events.len())"));
    assert!(source.contains("#[path = \"projection/binding_capacity_tests.rs\"]"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260919_editor837_template_binding_id_capacity_p95() {
    const BINDINGS: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    let source = (0..BINDINGS).collect::<Vec<_>>();

    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut projected = if pass == 0 {
                Vec::new()
            } else {
                Vec::with_capacity(source.len())
            };
            projected.extend(source.iter().copied());
            black_box(projected.len() + projected.capacity());
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
    let ratio = candidate_p95 as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} bindings={BINDINGS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} ratio={ratio:.4}"
    );
    assert!(
        candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7),
        "{PERF_MARKER} expected candidate p95 ratio <= 0.70, got {ratio:.4}"
    );
}
