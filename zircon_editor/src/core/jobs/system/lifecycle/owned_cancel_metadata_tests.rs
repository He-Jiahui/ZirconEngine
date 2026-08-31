use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ii_editor_cancel_metadata_moves_owned_label_and_token() {
    let observed_cancel = CancellationToken::default();
    let spec = EditorJobSpec::new(benchmark_label(), JobCategory::Misc)
        .with_cancel(observed_cancel.clone());
    let label_allocation = spec.label.as_ptr();

    let metadata = into_pending_cancel_metadata(spec);

    assert_eq!(metadata.label.as_ptr(), label_allocation);
    assert_eq!(Arc::strong_count(&metadata.label), 1);
    assert_eq!(metadata.category, JobCategory::Misc);
    metadata.cancel.cancel();
    assert!(observed_cancel.is_cancelled());
}

#[test]
fn optimization_batch_20260828ii_editor_pending_cancel_consumes_job_spec_metadata() {
    let source = include_str!("../lifecycle.rs");
    let cancel = source
        .split("pub fn cancel")
        .nth(1)
        .and_then(|body| body.split("pub fn shutdown").next())
        .expect("pending job cancellation implementation");
    let conversion = source
        .split("fn into_pending_cancel_metadata")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("pending cancel metadata conversion");

    assert!(cancel.contains("into_pending_cancel_metadata(pending.spec)"));
    assert!(!cancel.contains("pending.spec.label.clone()"));
    assert!(!cancel.contains("pending.spec.cancel.clone()"));
    assert!(conversion.contains("let EditorJobSpec"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ii_editor_owned_job_cancel_metadata_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 64 * 1024;

    black_box(legacy_cancel_metadata(benchmark_spec()));
    black_box(into_pending_cancel_metadata(benchmark_spec()));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_specs = benchmark_specs(ITERATIONS);
        let optimized_specs = benchmark_specs(ITERATIONS);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_specs(legacy_specs, legacy_cancel_metadata));
            optimized_samples.push(measure_specs(optimized_specs, into_pending_cancel_metadata));
        } else {
            optimized_samples.push(measure_specs(optimized_specs, into_pending_cancel_metadata));
            legacy_samples.push(measure_specs(legacy_specs, legacy_cancel_metadata));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR227_OWNED_JOB_CANCEL_METADATA_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_label() -> String {
    "pending-cancel-label/".repeat(8)
}

fn benchmark_spec() -> EditorJobSpec {
    EditorJobSpec::new(benchmark_label(), JobCategory::Misc)
}

fn benchmark_specs(count: usize) -> Vec<EditorJobSpec> {
    (0..count).map(|_| benchmark_spec()).collect()
}

fn legacy_cancel_metadata(spec: EditorJobSpec) -> PendingCancelMetadata {
    PendingCancelMetadata {
        label: spec.label.clone(),
        category: spec.category,
        cancel: spec.cancel.clone(),
    }
}

fn measure_specs(
    specs: Vec<EditorJobSpec>,
    mut convert: impl FnMut(EditorJobSpec) -> PendingCancelMetadata,
) -> u128 {
    let started = Instant::now();
    for spec in specs {
        black_box(convert(black_box(spec)));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
