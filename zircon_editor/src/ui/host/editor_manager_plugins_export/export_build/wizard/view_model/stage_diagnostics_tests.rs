use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const DIAGNOSTIC_COUNT: usize = 4_096;
const BENCH_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bg_export_stage_diagnostic_hash_merge_preserves_first_order() {
    let existing = vec!["existing".to_string(), "shared".to_string()];
    let additions = vec![
        "shared".to_string(),
        "new".to_string(),
        "new".to_string(),
        "tail".to_string(),
    ];

    assert_eq!(
        merge_unique_diagnostics(existing, &additions),
        ["existing", "shared", "new", "tail"]
    );
}

#[test]
fn optimization_batch_20260826bg_export_stage_diagnostic_hash_merge_eliminates_pairwise_work() {
    let comparisons =
        DIAGNOSTIC_COUNT * DIAGNOSTIC_COUNT + DIAGNOSTIC_COUNT * (DIAGNOSTIC_COUNT - 1) / 2;
    assert_eq!(comparisons, 25_163_776);

    let source = include_str!("../view_model.rs");
    let merge = source
        .split("fn merge_unique_diagnostics")
        .nth(1)
        .expect("diagnostic hash merge helper must exist")
        .split("fn stage_stdout_lines")
        .next()
        .expect("diagnostic hash merge helper must terminate");
    assert!(merge.contains("HashSet"));
    assert!(!merge.contains("diagnostics.contains"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826bg_export_stage_diagnostic_hash_merge_p95() {
    let existing = diagnostic_range("existing", BENCH_COUNT);
    let additions = diagnostic_range("additional", BENCH_COUNT);
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| {
                legacy_merge(existing.clone(), black_box(&additions))
            }));
            optimized.push(measure(|| {
                merge_unique_diagnostics(existing.clone(), black_box(&additions))
            }));
        } else {
            optimized.push(measure(|| {
                merge_unique_diagnostics(existing.clone(), black_box(&additions))
            }));
            baseline.push(measure(|| {
                legacy_merge(existing.clone(), black_box(&additions))
            }));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "EDITOR09_EXPORT_STAGE_DIAGNOSTIC_HASH_MERGE_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} pairwise_string_comparisons_before={} pairwise_string_comparisons_after=0 index_build_visits_after={BENCH_COUNT} hash_probes_after={BENCH_COUNT}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        BENCH_COUNT * BENCH_COUNT + BENCH_COUNT * (BENCH_COUNT - 1) / 2,
    );
    assert!(
        reduction >= 75.0,
        "expected at least 75% P95 reduction, got {reduction:.2}%"
    );
}

fn diagnostic_range(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("{prefix} diagnostic {index:05}"))
        .collect()
}

fn legacy_merge(mut diagnostics: Vec<String>, additions: &[String]) -> Vec<String> {
    for diagnostic in additions {
        if !diagnostics.contains(diagnostic) {
            diagnostics.push(diagnostic.clone());
        }
    }
    diagnostics
}

fn measure<T>(work: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(work());
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * percentile / 100]
}

fn percent_reduction(before: Duration, after: Duration) -> f64 {
    if before.is_zero() {
        return 0.0;
    }
    100.0 * (before.as_secs_f64() - after.as_secs_f64()) / before.as_secs_f64()
}
