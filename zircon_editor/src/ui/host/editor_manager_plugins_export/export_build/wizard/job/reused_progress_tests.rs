use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828im_editor_stage_record_reuses_progress_storage() {
    let source = benchmark_progress_state("windows-release");
    let mut target = benchmark_progress_state("debug");
    let allocation = target.snapshots().as_ptr();

    reuse_progress_state(&mut target, &source);

    assert_eq!(target.snapshots().as_ptr(), allocation);
    assert_eq!(target, source);
}

#[test]
fn optimization_batch_20260828im_editor_stage_execution_uses_progress_clone_from() {
    let source = include_str!("../job.rs");
    let record = source
        .split("pub fn record_stage_execution")
        .nth(1)
        .and_then(|body| body.split("pub fn finish_from_pipeline").next())
        .expect("stage execution recording implementation");
    let reuse = source
        .split("fn reuse_progress_state")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("progress reuse helper");

    assert!(record.contains("reuse_progress_state("));
    assert!(!record.contains("stage_execution.progress.clone()"));
    assert!(reuse.contains("target.clone_from(source)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828im_editor_reused_export_progress_state_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 64 * 1024;
    let source = benchmark_progress_state("windows-release");

    let mut warm = benchmark_progress_state("debug");
    legacy_update_progress(&mut warm, &source);
    reuse_progress_state(&mut warm, &source);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut legacy_target = benchmark_progress_state("debug");
        let mut optimized_target = benchmark_progress_state("debug");
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                legacy_update_progress(black_box(&mut legacy_target), black_box(&source));
            }
            started.elapsed().as_nanos()
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                reuse_progress_state(black_box(&mut optimized_target), black_box(&source));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
        black_box(legacy_target);
        black_box(optimized_target);
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR231_REUSED_EXPORT_PROGRESS_STATE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_progress_state(profile: &str) -> ExportWizardProgressState {
    let mut progress = ExportWizardProgressState::new();
    progress.push_stdout_line(&format!("zircon_export stage=Report profile={profile}"));
    progress.push_stdout_line("report=D:\\export\\stages\\report.json");
    progress.push_stdout_line("artifact=D:\\export\\bundle\\zircon-runtime.zip");
    progress.push_stdout_line("warning: representative export diagnostic");
    progress.push_stdout_line(r#""fatal": false,"#);
    progress
}

fn legacy_update_progress(
    target: &mut ExportWizardProgressState,
    source: &ExportWizardProgressState,
) {
    *target = source.clone();
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
