use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ENTRY_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bg_native_string_list_hash_dedup_preserves_first_order() {
    assert_eq!(
        parse_native_string_list(" alpha, beta; alpha\ngamma,, beta ; delta "),
        ["alpha", "beta", "gamma", "delta"]
    );
}

#[test]
fn optimization_batch_20260826bg_native_string_list_hash_dedup_eliminates_pairwise_work() {
    assert_eq!(ENTRY_COUNT * (ENTRY_COUNT - 1) / 2, 8_386_560);

    let source = include_str!("../native_strings.rs");
    let parser = source
        .split("pub(super) fn parse_native_string_list")
        .nth(1)
        .expect("native string list parser must exist")
        .split("#[cfg(test)]")
        .next()
        .expect("native string list parser must terminate");
    assert!(parser.contains("HashSet"));
    assert!(!parser.contains("entries.iter().any"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826bg_native_string_list_hash_dedup_p95() {
    let input = native_string_list(ENTRY_COUNT);
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| legacy_parse(black_box(&input))));
            optimized.push(measure(|| parse_native_string_list(black_box(&input))));
        } else {
            optimized.push(measure(|| parse_native_string_list(black_box(&input))));
            baseline.push(measure(|| legacy_parse(black_box(&input))));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "RUNTIME07_NATIVE_STRING_LIST_HASH_DEDUP_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} pairwise_string_comparisons_before=8386560 pairwise_string_comparisons_after=0 hash_probes_after={ENTRY_COUNT}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        reduction >= 75.0,
        "expected at least 75% P95 reduction, got {reduction:.2}%"
    );
}

fn native_string_list(count: usize) -> String {
    (0..count)
        .rev()
        .map(|index| format!("capability.{index:05}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn legacy_parse(value: &str) -> Vec<String> {
    let mut entries = Vec::new();
    for entry in value
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if !entries.iter().any(|existing| existing == entry) {
            entries.push(entry.to_string());
        }
    }
    entries
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
