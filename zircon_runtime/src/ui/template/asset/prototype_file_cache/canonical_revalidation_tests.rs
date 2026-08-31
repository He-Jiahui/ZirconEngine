use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::UiPrototypeFileStoreCacheKey;

const SOURCE_COUNT: usize = 512;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bk_prototype_cache_canonical_revalidation_preserves_key() {
    let canonical = source_path();

    assert_eq!(
        UiPrototypeFileStoreCacheKey::from_paths(std::slice::from_ref(&canonical)),
        UiPrototypeFileStoreCacheKey::from_canonical_paths(std::slice::from_ref(&canonical))
    );
}

#[test]
fn optimization_batch_20260826bk_prototype_cache_canonical_revalidation_skips_resolve() {
    const SOURCE: &str = include_str!("../prototype_file_cache.rs");
    let revalidation = SOURCE
        .split("let current_source_key =")
        .nth(1)
        .and_then(|tail| tail.split(';').next())
        .expect("cache-hit source revalidation expression");
    let canonical_constructor = SOURCE
        .split("fn from_canonical_paths")
        .nth(1)
        .and_then(|tail| tail.split("fn from_path").next())
        .expect("canonical path key constructor");

    assert_eq!(SOURCE_COUNT, 512);
    assert!(revalidation.contains("from_canonical_paths(&entry.source_paths)"));
    assert!(!canonical_constructor.contains("canonicalize()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bk_prototype_cache_canonical_revalidation_p95() {
    let canonical = source_path();
    let paths = vec![canonical; SOURCE_COUNT];

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || UiPrototypeFileStoreCacheKey::from_paths(black_box(&paths)),
        || UiPrototypeFileStoreCacheKey::from_canonical_paths(black_box(&paths)),
    );
    assert_eq!(
        UiPrototypeFileStoreCacheKey::from_paths(&paths),
        UiPrototypeFileStoreCacheKey::from_canonical_paths(&paths)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME11A_PROTOTYPE_CACHE_CANONICAL_REVALIDATION_BENCH_V1 sources={SOURCE_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_redundant_canonicalize_calls={SOURCE_COUNT} optimized_redundant_canonicalize_calls=0 metadata_freshness_checks={SOURCE_COUNT} deterministic_redundant_canonicalize_reduction_percent=100.0000 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be at least 20% below legacy P95 {legacy_p95}ns"
    );
}

fn source_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/ui/template/asset/prototype_file_cache.rs")
        .canonicalize()
        .expect("prototype cache source path")
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> UiPrototypeFileStoreCacheKey,
    mut optimized: impl FnMut() -> UiPrototypeFileStoreCacheKey,
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

fn benchmark_sample(operation: &mut impl FnMut() -> UiPrototypeFileStoreCacheKey) -> u128 {
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
