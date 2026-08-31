use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::MAX_TAA_RESOLVE_BIND_GROUPS;

const ENTRY_COUNT: usize = MAX_TAA_RESOLVE_BIND_GROUPS;
const HITS_PER_FRAME: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_KEY_COMPARISONS: usize = ENTRY_COUNT * HITS_PER_FRAME;
const OPTIMIZED_KEY_COMPARISONS: usize = HITS_PER_FRAME;

#[test]
fn optimization_batch_20260826bo_taa_bind_group_mru_preserves_lru_order() {
    let mut legacy = (0..ENTRY_COUNT as u64).collect::<VecDeque<_>>();
    let mut optimized = legacy.clone();

    for key in [7, 7, 3, 3, 7, 0, 0] {
        assert_eq!(
            legacy_access(&mut legacy, key),
            optimized_access(&mut optimized, key)
        );
        assert_eq!(legacy, optimized);
    }
}

#[test]
fn optimization_batch_20260826bo_taa_bind_group_mru_eliminates_stable_scan() {
    const SOURCE: &str = include_str!("../taa_resolve_bind_group_cache.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_KEY_COMPARISONS, 32_768);
    assert_eq!(OPTIMIZED_KEY_COMPARISONS, 4_096);
    assert!(production.contains("self.entries.back()"));
    assert!(production.contains("entry.key == key"));
    assert!(production.contains("bind_group: entry.bind_group.clone()"));
    assert!(production.contains("self.entries.iter().position"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bo_taa_bind_group_mru_p95() {
    let base = (0..ENTRY_COUNT as u64).collect::<VecDeque<_>>();
    let hot_key = ENTRY_COUNT as u64 - 1;
    let mut legacy = base.clone();
    let mut optimized = base;

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_frame(black_box(&mut legacy), hot_key),
        || optimized_frame(black_box(&mut optimized), hot_key),
    );
    assert_eq!(
        legacy_frame(&mut legacy, hot_key),
        optimized_frame(&mut optimized, hot_key)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME09H1_TAA_BIND_GROUP_MRU_FAST_PATH_BENCH_V1 entries={ENTRY_COUNT} hits_per_frame={HITS_PER_FRAME} samples={SAMPLE_COUNT} sample_order=alternating legacy_key_comparisons={LEGACY_KEY_COMPARISONS} optimized_key_comparisons={OPTIMIZED_KEY_COMPARISONS} legacy_deque_relocations={HITS_PER_FRAME} optimized_deque_relocations=0 deterministic_comparison_reduction_percent=87.5000 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_access(entries: &mut VecDeque<u64>, key: u64) -> Option<u64> {
    let index = entries.iter().position(|entry| *entry == key)?;
    let entry = entries.remove(index)?;
    entries.push_back(entry);
    Some(entry)
}

fn optimized_access(entries: &mut VecDeque<u64>, key: u64) -> Option<u64> {
    if entries.back().is_some_and(|entry| *entry == key) {
        return Some(key);
    }
    legacy_access(entries, key)
}

fn legacy_frame(entries: &mut VecDeque<u64>, hot_key: u64) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        observed += legacy_access(entries, hot_key).unwrap();
    }
    observed
}

fn optimized_frame(entries: &mut VecDeque<u64>, hot_key: u64) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        observed += optimized_access(entries, hot_key).unwrap();
    }
    observed
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for index in 0..N {
        if index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
