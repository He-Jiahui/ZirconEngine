use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_COUNT: usize = 17;
const TRACK_COUNT: usize = 1_024;
const ITERATIONS: usize = 128;

fn legacy_duplicate_count(paths: &[String]) -> usize {
    let mut seen = BTreeSet::new();
    paths
        .iter()
        .filter(|path| !seen.insert(path.as_str()))
        .count()
}

fn optimized_duplicate_count(paths: &[String]) -> usize {
    let mut seen = HashSet::with_capacity(paths.len());
    paths
        .iter()
        .filter(|path| !seen.insert(path.as_str()))
        .count()
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn optimization_batch_hi_runtime594_hash_duplicate_detection_preserves_results() {
    let paths = (0..TRACK_COUNT)
        .map(|index| format!("Transform.channel_{:04}", index % 256))
        .collect::<Vec<_>>();

    assert_eq!(
        optimized_duplicate_count(&paths),
        legacy_duplicate_count(&paths)
    );
    assert!(optimized_duplicate_count(&paths) > 0);
}

#[test]
fn optimization_batch_hi_runtime594_sequence_compile_uses_hash_membership() {
    let production = include_str!("../compile.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();

    assert!(production.contains("HashSet::with_capacity(binding.tracks.len())"));
    assert!(!production.contains("BTreeSet"));
    assert!(production.contains("let mut diagnostics = Vec::new()"));
    assert!(!production.contains("Vec::with_capacity(asset.bindings.len())"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn optimization_batch_hi_runtime594_hash_duplicate_detection_bench() {
    let paths = (0..TRACK_COUNT)
        .map(|index| format!("Transform.channel_{:04}", index % 256))
        .collect::<Vec<_>>();
    let expected = legacy_duplicate_count(&paths);
    assert_eq!(optimized_duplicate_count(&paths), expected);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_duplicate_count(black_box(&paths)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_duplicate_count(black_box(&paths)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME594_ANIMATION_DUPLICATE_TRACK_HASH_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} track_count={} membership=btree->hash capacity=0->{}",
        legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS, TRACK_COUNT, TRACK_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90),
        "optimized duplicate detection P95 must be at most 90% of legacy P95"
    );
}
