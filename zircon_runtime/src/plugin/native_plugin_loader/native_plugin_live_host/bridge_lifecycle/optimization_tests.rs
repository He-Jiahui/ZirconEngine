use std::hint::black_box;
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 512;
const ENTRY_COUNT: usize = 4_096;
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

fn fixture_values(prefix: &str) -> Vec<String> {
    (0..ENTRY_COUNT)
        .map(|index| format!("{prefix}.{:04}", index % UNIQUE_ENTRY_COUNT))
        .collect()
}

fn legacy_dedup(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn optimized_dedup(mut values: Vec<String>) -> Vec<String> {
    values.sort_unstable();
    values.dedup();
    values
}

fn legacy_load_finalize(
    loaded_plugin_ids: Vec<String>,
    diagnostics: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    (legacy_dedup(loaded_plugin_ids), legacy_dedup(diagnostics))
}

fn optimized_load_finalize(
    loaded_plugin_ids: Vec<String>,
    diagnostics: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    (
        optimized_dedup(loaded_plugin_ids),
        optimized_dedup(diagnostics),
    )
}

fn legacy_hot_update_finalize(
    manifest_plugin_ids: Vec<String>,
    runtime_plugin_ids: Vec<String>,
    skipped_plugin_ids: Vec<String>,
    loaded_plugin_ids: Vec<String>,
    diagnostics: Vec<String>,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    (
        legacy_dedup(manifest_plugin_ids),
        legacy_dedup(runtime_plugin_ids),
        legacy_dedup(skipped_plugin_ids),
        legacy_dedup(loaded_plugin_ids),
        legacy_dedup(diagnostics),
    )
}

fn optimized_hot_update_finalize(
    manifest_plugin_ids: Vec<String>,
    runtime_plugin_ids: Vec<String>,
    skipped_plugin_ids: Vec<String>,
    loaded_plugin_ids: Vec<String>,
    diagnostics: Vec<String>,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    (
        optimized_dedup(manifest_plugin_ids),
        optimized_dedup(runtime_plugin_ids),
        optimized_dedup(skipped_plugin_ids),
        optimized_dedup(loaded_plugin_ids),
        optimized_dedup(diagnostics),
    )
}

#[test]
fn runtime58_unstable_live_host_output_dedup_preserves_results() {
    let loaded = fixture_values("loaded");
    let diagnostics = fixture_values("diagnostic");
    assert_eq!(
        optimized_load_finalize(loaded.clone(), diagnostics.clone()),
        legacy_load_finalize(loaded, diagnostics)
    );

    let manifest = fixture_values("manifest");
    let runtime = fixture_values("runtime");
    let skipped = fixture_values("skipped");
    let loaded = fixture_values("loaded");
    let diagnostics = fixture_values("diagnostic");
    assert_eq!(
        optimized_hot_update_finalize(
            manifest.clone(),
            runtime.clone(),
            skipped.clone(),
            loaded.clone(),
            diagnostics.clone(),
        ),
        legacy_hot_update_finalize(manifest, runtime, skipped, loaded, diagnostics)
    );
}

#[test]
fn runtime58_unstable_live_host_output_source_contract() {
    let bridge_source = include_str!("../bridge_lifecycle.rs");
    let hot_update_source = include_str!("../hot_update_application.rs");
    let loading_source = include_str!("../loading.rs");

    assert_eq!(
        bridge_source.matches("sort_unstable();").count(),
        3,
        "bridge lifecycle should use unstable sort for all deduped diagnostics"
    );
    assert_eq!(
        hot_update_source.matches("sort_unstable();").count(),
        4,
        "hot update should use unstable sort for all deduped ID lists"
    );
    assert_eq!(
        loading_source.matches("sort_unstable();").count(),
        2,
        "loading should use unstable sort for IDs and diagnostics"
    );
    assert!(!bridge_source.contains(".sort();"));
    assert!(!hot_update_source.contains(".sort();"));
    assert!(!loading_source.contains(".sort();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_unstable_bridge_lifecycle_diagnostics_bench() {
    let diagnostics = fixture_values("diagnostic");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_dedup(diagnostics.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_dedup(diagnostics.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_UNSTABLE_BRIDGE_LIFECYCLE_DIAGNOSTICS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=3->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(optimized_dedup(diagnostics).len(), UNIQUE_ENTRY_COUNT);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_unstable_load_hot_update_outputs_bench() {
    let manifest = fixture_values("manifest");
    let runtime = fixture_values("runtime");
    let skipped = fixture_values("skipped");
    let loaded = fixture_values("loaded");
    let diagnostics = fixture_values("diagnostic");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_hot_update_finalize(
                manifest.clone(),
                runtime.clone(),
                skipped.clone(),
                loaded.clone(),
                diagnostics.clone(),
            ));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_hot_update_finalize(
                manifest.clone(),
                runtime.clone(),
                skipped.clone(),
                loaded.clone(),
                diagnostics.clone(),
            ));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_UNSTABLE_LOAD_HOT_UPDATE_OUTPUTS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} lists=5 entries_per_list={} unique_entries_per_list={} stable_sorts=6->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(
        optimized_hot_update_finalize(manifest, runtime, skipped, loaded, diagnostics)
            .0
            .len(),
        UNIQUE_ENTRY_COUNT
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
