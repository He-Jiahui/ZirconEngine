use std::hint::black_box;
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 512;
const ENTRY_COUNT: usize = 2_048;
const UNIQUE_ENTRY_COUNT: usize = ENTRY_COUNT / 2;

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

fn fixture_capabilities(prefix: &str) -> Vec<String> {
    (0..ENTRY_COUNT)
        .map(|index| format!("{prefix}.{:04}", index % UNIQUE_ENTRY_COUNT))
        .collect()
}

fn legacy_unique(mut capabilities: Vec<String>) -> Vec<String> {
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

fn optimized_unique(mut capabilities: Vec<String>) -> Vec<String> {
    capabilities.sort_unstable();
    capabilities.dedup();
    capabilities
}

#[test]
fn editor06_unstable_capability_dedup_preserves_sorted_unique_results() {
    let capabilities = fixture_capabilities("editor.capability");
    assert_eq!(
        optimized_unique(capabilities.clone()),
        legacy_unique(capabilities)
    );
    assert_eq!(
        optimized_unique(fixture_capabilities("editor.capability")).len(),
        UNIQUE_ENTRY_COUNT
    );
}

#[test]
fn editor06_unstable_capability_dedup_source_contract() {
    let snapshot_source = include_str!("../catalog_snapshot.rs");
    let production = snapshot_source
        .split_once("#[cfg(test)]")
        .expect("optimization test module should follow production")
        .0;
    assert!(production.contains("package_ids.sort_unstable();"));
    assert!(!production.contains("package_ids.sort();"));

    let projection_source = include_str!("../projection.rs");
    assert!(projection_source.contains("capabilities.sort_unstable();"));
    assert!(!projection_source.contains("capabilities.sort();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_unstable_plugin_capability_index_bench() {
    let capabilities = fixture_capabilities("editor.capability");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_unique(capabilities.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_unique(capabilities.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_UNSTABLE_PLUGIN_CAPABILITY_INDEX_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(optimized_unique(capabilities).len(), UNIQUE_ENTRY_COUNT);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_unstable_plugin_capability_projection_bench() {
    let capabilities = fixture_capabilities("editor.capability");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_unique(capabilities.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_unique(capabilities.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_UNSTABLE_PLUGIN_CAPABILITY_PROJECTION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(optimized_unique(capabilities).len(), UNIQUE_ENTRY_COUNT);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
