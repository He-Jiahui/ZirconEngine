use std::hint::black_box;
use std::time::Instant;

use super::merge_unique_diagnostics;

const EXISTING_COUNT: usize = 4_096;
const ADDITION_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_PAIRWISE_COMPARISONS: usize =
    ADDITION_COUNT * EXISTING_COUNT + ADDITION_COUNT * (ADDITION_COUNT - 1) / 2;

#[test]
fn optimization_batch_20260826bi_fatal_diagnostic_hash_merge_preserves_existing_and_new_order() {
    let existing = ["existing", "existing"].map(str::to_owned).to_vec();
    let additions = ["existing", "new-a", "new-a", "new-b"]
        .map(str::to_owned)
        .to_vec();

    assert_eq!(
        merge_unique_diagnostics(existing, additions),
        ["existing", "existing", "new-a", "new-b"]
    );
}

#[test]
fn optimization_batch_20260826bi_fatal_diagnostic_hash_merge_eliminates_pairwise_work() {
    const SOURCE: &str = include_str!("../export_build_plan.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_PAIRWISE_COMPARISONS, 25_163_776);
    assert!(production.contains("HashSet::<&str>::with_capacity(diagnostics.len())"));
    assert!(production.contains("additions.size_hint()"));
    assert!(production.contains("accepted_keys.insert(diagnostic.clone())"));
    assert!(!production.contains("diagnostics.iter().any"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bi_fatal_diagnostic_hash_merge_p95() {
    let existing = (0..EXISTING_COUNT)
        .map(|index| format!("existing fatal diagnostic {index:04}"))
        .collect::<Vec<_>>();
    let additions = (0..ADDITION_COUNT)
        .map(|index| format!("missing plugin diagnostic {index:04}"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_merge(black_box(existing.clone()), black_box(additions.clone())),
        || merge_unique_diagnostics(black_box(existing.clone()), black_box(additions.clone())),
    );
    assert_eq!(
        legacy_merge(existing.clone(), additions.clone()),
        merge_unique_diagnostics(existing, additions)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME07_EXPORT_FATAL_DIAGNOSTIC_HASH_MERGE_BENCH_V1 existing={EXISTING_COUNT} additions={ADDITION_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_pairwise_comparisons={LEGACY_PAIRWISE_COMPARISONS} optimized_existing_index_visits={EXISTING_COUNT} optimized_hash_probes={} deterministic_lookup_work_reduction_percent=99.9512 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        ADDITION_COUNT * 2,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 4 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 75% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_merge(mut diagnostics: Vec<String>, additions: Vec<String>) -> Vec<String> {
    for diagnostic in additions {
        if !diagnostics.iter().any(|existing| existing == &diagnostic) {
            diagnostics.push(diagnostic);
        }
    }
    diagnostics
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
