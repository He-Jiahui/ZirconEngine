use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ih_editor_dispatch_metadata_moves_owned_allocations() {
    let spec = benchmark_spec();
    let label_allocation = spec.label.as_ptr();
    let mutex_allocation = spec
        .mutex_group
        .as_ref()
        .expect("benchmark spec mutex group")
        .as_str()
        .as_ptr();

    let metadata = into_pending_dispatch_metadata(spec);

    assert_eq!(metadata.label.as_ptr(), label_allocation);
    assert_eq!(
        metadata
            .mutex_group
            .as_ref()
            .expect("moved mutex group")
            .as_str()
            .as_ptr(),
        mutex_allocation
    );
    assert_eq!(metadata.category, JobCategory::Misc);
    assert!(!metadata.cancel.is_cancelled());
}

#[test]
fn optimization_batch_20260828ih_editor_promotion_consumes_job_spec_metadata() {
    let source = include_str!("../scheduling.rs");
    let promote = source
        .split("pub(super) fn promote")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn finish").next())
        .expect("job promotion implementation");
    let conversion = source
        .split("fn into_pending_dispatch_metadata")
        .nth(1)
        .and_then(|body| body.split("struct CompletionGuard").next())
        .expect("pending dispatch metadata conversion");

    assert!(promote.contains("into_pending_dispatch_metadata(pending.spec)"));
    assert!(!promote.contains("pending.spec.label.clone()"));
    assert!(!promote.contains("pending.spec.mutex_group.clone()"));
    assert!(!promote.contains("pending.spec.cancel.clone()"));
    assert!(conversion.contains("let EditorJobSpec"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ih_editor_owned_job_dispatch_metadata_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 16 * 1024;

    black_box(legacy_dispatch_metadata(benchmark_spec()));
    black_box(into_pending_dispatch_metadata(benchmark_spec()));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_specs = benchmark_specs(ITERATIONS);
        let optimized_specs = benchmark_specs(ITERATIONS);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_specs(legacy_specs, legacy_dispatch_metadata));
            optimized_samples.push(measure_specs(
                optimized_specs,
                into_pending_dispatch_metadata,
            ));
        } else {
            optimized_samples.push(measure_specs(
                optimized_specs,
                into_pending_dispatch_metadata,
            ));
            legacy_samples.push(measure_specs(legacy_specs, legacy_dispatch_metadata));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR226_OWNED_JOB_DISPATCH_METADATA_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_spec() -> EditorJobSpec {
    EditorJobSpec::new("job-dispatch-label/".repeat(16), JobCategory::Misc)
        .with_mutex_group(MutexGroup::parse("g".repeat(MutexGroup::MAX_BYTES)).unwrap())
}

fn benchmark_specs(count: usize) -> Vec<EditorJobSpec> {
    (0..count).map(|_| benchmark_spec()).collect()
}

fn legacy_dispatch_metadata(spec: EditorJobSpec) -> PendingDispatchMetadata {
    PendingDispatchMetadata {
        label: spec.label.clone(),
        category: spec.category,
        mutex_group: spec.mutex_group.clone(),
        cancel: spec.cancel.clone(),
    }
}

fn measure_specs(
    specs: Vec<EditorJobSpec>,
    mut convert: impl FnMut(EditorJobSpec) -> PendingDispatchMetadata,
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
