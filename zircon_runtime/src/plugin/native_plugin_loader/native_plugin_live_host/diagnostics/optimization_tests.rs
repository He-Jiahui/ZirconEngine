use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{combine_diagnostics, sorted_unique_diagnostics};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 512;
const DIAGNOSTIC_COUNT: usize = 4_096;
const GROUP_COUNT: usize = 4;
const GROUP_DIAGNOSTIC_COUNT: usize = DIAGNOSTIC_COUNT / GROUP_COUNT;
const UNIQUE_DIAGNOSTIC_COUNT: usize = DIAGNOSTIC_COUNT / 2;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn fixture_diagnostics(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("{prefix}.{:04}", index % UNIQUE_DIAGNOSTIC_COUNT))
        .collect()
}

fn legacy_sorted_unique(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn legacy_combine<const N: usize>(diagnostic_groups: [Vec<String>; N]) -> Vec<String> {
    legacy_sorted_unique(diagnostic_groups.into_iter().flatten().collect::<Vec<_>>())
}

#[test]
fn runtime58_diagnostics_optimization_preserves_sorted_unique_results() {
    let diagnostics = fixture_diagnostics("diagnostic", DIAGNOSTIC_COUNT);
    assert_eq!(
        sorted_unique_diagnostics(diagnostics.clone()),
        legacy_sorted_unique(diagnostics)
    );

    let groups = std::array::from_fn(|index| {
        fixture_diagnostics(&format!("group-{index}"), GROUP_DIAGNOSTIC_COUNT)
    });
    let combined = combine_diagnostics(groups.clone());
    assert_eq!(combined, legacy_combine(groups));
    assert_eq!(combined.len(), GROUP_COUNT * GROUP_DIAGNOSTIC_COUNT);
}

#[test]
fn runtime58_diagnostics_optimization_source_contract() {
    let source = include_str!("../diagnostics.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("optimization test module should follow production")
        .0;
    assert!(production.contains("let capacity = diagnostic_groups.iter().map(Vec::len).sum();"));
    assert!(production.contains("Vec::with_capacity(capacity)"));
    assert!(production.contains("diagnostics.sort_unstable();"));
    assert!(!production.contains("diagnostics.sort();"));
    assert!(!production.contains("diagnostic_groups.into_iter().flatten().collect"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_unstable_diagnostics_sort_bench() {
    let diagnostics = fixture_diagnostics("diagnostic", DIAGNOSTIC_COUNT);
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_sorted_unique(diagnostics.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(sorted_unique_diagnostics(diagnostics.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_UNSTABLE_DIAGNOSTICS_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        DIAGNOSTIC_COUNT,
        UNIQUE_DIAGNOSTIC_COUNT,
    );
    assert_eq!(
        sorted_unique_diagnostics(diagnostics).len(),
        UNIQUE_DIAGNOSTIC_COUNT
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_preallocated_diagnostics_merge_bench() {
    let groups = std::array::from_fn(|index| {
        fixture_diagnostics(&format!("group-{index}"), GROUP_DIAGNOSTIC_COUNT)
    });
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_combine(groups.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(combine_diagnostics(groups.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_PREALLOCATED_DIAGNOSTICS_MERGE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} groups={} diagnostics_per_group={} total_entries={} preallocated_capacity=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        GROUP_COUNT,
        GROUP_DIAGNOSTIC_COUNT,
        DIAGNOSTIC_COUNT,
        DIAGNOSTIC_COUNT,
    );
    assert_eq!(combine_diagnostics(groups).len(), DIAGNOSTIC_COUNT);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 90,
        "optimized p95 should be at most 90% of legacy p95"
    );
}
