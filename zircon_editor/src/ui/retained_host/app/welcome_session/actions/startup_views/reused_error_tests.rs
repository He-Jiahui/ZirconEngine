use std::fmt;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828if_editor_startup_error_reuses_status_buffer() {
    let mut status_message = String::with_capacity(16 * 1024);
    status_message.push_str("previous startup status");
    let allocation = status_message.as_ptr();
    let error = StructuredStartupError { segments: 64 };

    let status_line = update_startup_error(&mut status_message, &error);

    assert_eq!(status_message.as_ptr(), allocation);
    assert_eq!(status_line, status_message);
    assert!(status_line.contains("startup-stage-63"));
}

#[test]
fn optimization_batch_20260828if_editor_startup_workbench_formats_error_once() {
    let source = include_str!("../startup_views.rs");
    let workbench = source
        .split("pub(super) fn open_startup_workbench")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn open_startup_view").next())
        .expect("startup workbench implementation");
    let update = source
        .split("fn update_startup_error")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("startup error update implementation");

    assert!(workbench.contains("update_startup_error("));
    assert!(!workbench.contains("error.to_string()"));
    assert_eq!(update.matches("error.to_string()").count(), 1);
    assert!(update.contains("status_message.clone_from(&formatted)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828if_editor_reused_startup_error_format_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 2 * 1024;
    let error = StructuredStartupError { segments: 128 };

    let mut warm = seeded_status();
    black_box(legacy_startup_error(&mut warm, &error));
    black_box(update_startup_error(&mut warm, &error));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let mut status_message = seeded_status();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let status_line =
                    legacy_startup_error(black_box(&mut status_message), black_box(&error));
                black_box(status_line.clone());
                black_box(status_line);
            }
            black_box(status_message);
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let mut status_message = seeded_status();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let status_line =
                    update_startup_error(black_box(&mut status_message), black_box(&error));
                black_box(status_line.clone());
                black_box(status_line);
            }
            black_box(status_message);
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR224_REUSED_STARTUP_ERROR_FORMAT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

struct StructuredStartupError {
    segments: usize,
}

impl fmt::Display for StructuredStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in 0..self.segments {
            write!(
                formatter,
                "startup-stage-{segment}: project manifest dependency resolution failed; "
            )?;
        }
        Ok(())
    }
}

fn seeded_status() -> String {
    let mut status = String::with_capacity(16 * 1024);
    status.push_str("previous startup status");
    status
}

fn legacy_startup_error(status_message: &mut String, error: &StructuredStartupError) -> String {
    *status_message = error.to_string();
    error.to_string()
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
