use std::hint::black_box;
use std::time::Instant;

use super::dedupe;

const DIAGNOSTIC_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_PAIRWISE_COMPARISONS: usize = DIAGNOSTIC_COUNT * (DIAGNOSTIC_COUNT - 1) / 2;

#[test]
fn optimization_batch_20260826bh_export_validate_hash_dedup_preserves_first_order() {
    let diagnostics = ["alpha", "beta", "alpha", "gamma", "beta"]
        .map(str::to_owned)
        .to_vec();

    assert_eq!(dedupe(diagnostics), ["alpha", "beta", "gamma"]);
}

#[test]
fn optimization_batch_20260826bh_export_validate_hash_dedup_eliminates_pairwise_work() {
    const SOURCE: &str = include_str!("../export_validate_report.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_PAIRWISE_COMPARISONS, 8_386_560);
    assert!(production.contains("HashSet::<&str>::with_capacity(values.len())"));
    assert!(production.contains("seen.insert(value.as_str())"));
    assert!(!production.contains("deduped.iter().any"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bh_export_validate_hash_dedup_p95() {
    let diagnostics = (0..DIAGNOSTIC_COUNT)
        .map(|index| format!("export diagnostic {index:04}: plugin capability unavailable"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_dedupe(black_box(diagnostics.clone())),
        || dedupe(black_box(diagnostics.clone())),
    );
    assert_eq!(legacy_dedupe(diagnostics.clone()), dedupe(diagnostics));

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME07_EXPORT_VALIDATE_DIAGNOSTIC_HASH_DEDUP_BENCH_V1 diagnostics={DIAGNOSTIC_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_pairwise_comparisons={LEGACY_PAIRWISE_COMPARISONS} optimized_hash_probes={DIAGNOSTIC_COUNT} deterministic_probe_reduction_percent=99.9512 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 4 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 75% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_dedupe(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::with_capacity(values.len());
    for value in values {
        if !deduped.iter().any(|existing| existing == &value) {
            deduped.push(value);
        }
    }
    deduped
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> Vec<String>,
    mut optimized: impl FnMut() -> Vec<String>,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> Vec<String>) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
