use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ij_editor_welcome_draft_reuses_string_capacity() {
    let mut target = String::with_capacity(4 * 1024);
    target.push_str("previous draft value");
    let allocation = target.as_ptr();
    let value = "new-project-name/".repeat(32);

    update_draft_text(&mut target, &value);

    assert_eq!(target.as_ptr(), allocation);
    assert_eq!(target, value);
}

#[test]
fn optimization_batch_20260828ij_editor_welcome_updates_use_reused_draft_text() {
    let source = include_str!("../draft.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("welcome draft production implementation");

    assert_eq!(production.matches("update_draft_text(").count(), 3);
    assert!(!production.contains("value.to_string()"));
    assert!(production.contains("target.clear()"));
    assert!(production.contains("target.push_str(value)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ij_editor_reused_welcome_draft_text_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 64 * 1024;
    let value = "welcome-draft/".repeat(18);

    let mut warm = seeded_draft();
    legacy_update_draft_text(&mut warm, &value);
    update_draft_text(&mut warm, &value);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut legacy_target = seeded_draft();
        let mut optimized_target = seeded_draft();
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                legacy_update_draft_text(black_box(&mut legacy_target), black_box(value.as_str()));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                update_draft_text(black_box(&mut optimized_target), black_box(value.as_str()));
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
        "EDITOR228_REUSED_WELCOME_DRAFT_TEXT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn seeded_draft() -> String {
    let mut draft = String::with_capacity(4 * 1024);
    draft.push_str("previous draft value");
    draft
}

fn legacy_update_draft_text(target: &mut String, value: &str) {
    *target = value.to_string();
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
