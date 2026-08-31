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

fn legacy_report_dedup(
    mut diagnostics: Vec<String>,
    mut skipped_plugin_ids: Vec<String>,
) -> (usize, usize) {
    diagnostics.sort();
    diagnostics.dedup();
    skipped_plugin_ids.sort();
    skipped_plugin_ids.dedup();
    (diagnostics.len(), skipped_plugin_ids.len())
}

fn optimized_report_dedup(
    mut diagnostics: Vec<String>,
    mut skipped_plugin_ids: Vec<String>,
) -> (usize, usize) {
    diagnostics.sort_unstable();
    diagnostics.dedup();
    skipped_plugin_ids.sort_unstable();
    skipped_plugin_ids.dedup();
    (diagnostics.len(), skipped_plugin_ids.len())
}

fn legacy_component_id_dedup(mut component_type_ids: Vec<String>) -> usize {
    component_type_ids.sort();
    component_type_ids.dedup();
    component_type_ids.len()
}

fn optimized_component_id_dedup(mut component_type_ids: Vec<String>) -> usize {
    component_type_ids.sort_unstable();
    component_type_ids.dedup();
    component_type_ids.len()
}

#[test]
fn runtime58_unstable_replay_dedup_preserves_sorted_unique_output() {
    let diagnostics = fixture_values("diagnostic");
    let skipped_plugin_ids = fixture_values("plugin");
    assert_eq!(
        legacy_report_dedup(diagnostics.clone(), skipped_plugin_ids.clone()),
        optimized_report_dedup(diagnostics, skipped_plugin_ids)
    );
    assert_eq!(
        legacy_component_id_dedup(fixture_values("component")),
        UNIQUE_ENTRY_COUNT
    );
}

#[test]
fn runtime58_unstable_replay_dedup_source_contract() {
    let replay_source = include_str!("../registration_replay.rs");
    assert_eq!(replay_source.matches(".sort_unstable();").count(), 3);
    assert!(!replay_source.contains(".sort();"));

    let bridge_source = include_str!("../bridge_methods.rs");
    assert!(bridge_source.contains(".sort_unstable();"));
    assert!(!bridge_source.contains(".sort();"));
}

#[test]
fn runtime58_registration_replay_missing_context_fails_closed_source_contract() {
    let replay_source = include_str!("../registration_replay.rs");
    assert!(!replay_source
        .contains(".expect(\"non-empty registration manifest must retain a replay context\")"));
    assert!(replay_source.contains("generation.replay_context.as_deref().ok_or_else(||"));
    assert!(replay_source.contains("NativePluginRegistrationReplayError::BridgeCallScope"));
}

#[test]
fn runtime58_registration_replay_prepared_system_count_mismatch_fails_closed_source_contract() {
    let replay_source = include_str!("../registration_replay.rs");
    let count_guard = "manifest.systems.len() != generation.prepared_systems.len()";
    assert!(replay_source.contains(count_guard));
    assert!(!replay_source
        .contains("debug_assert_eq!(manifest.systems.len(), generation.prepared_systems.len())"));
    assert!(
        replay_source
            .find(count_guard)
            .expect("count guard should exist")
            < replay_source
                .find("if manifest.systems.is_empty()")
                .expect("empty manifest guard should exist")
    );
    assert!(
        replay_source
            .find(count_guard)
            .expect("count guard should exist")
            < replay_source
                .find("let known_component_ids = generation")
                .expect("known component ids should be assembled")
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_unstable_diagnostic_dedup_bench() {
    let diagnostics = fixture_values("diagnostic");
    let skipped_plugin_ids = fixture_values("plugin");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_report_dedup(
                diagnostics.clone(),
                skipped_plugin_ids.clone(),
            ));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_report_dedup(
                diagnostics.clone(),
                skipped_plugin_ids.clone(),
            ));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_UNSTABLE_DIAGNOSTIC_DEDUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=2->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT * 2,
        UNIQUE_ENTRY_COUNT * 2,
    );
    assert_eq!(
        optimized_report_dedup(diagnostics, skipped_plugin_ids),
        (UNIQUE_ENTRY_COUNT, UNIQUE_ENTRY_COUNT)
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_unstable_component_id_dedup_bench() {
    let component_type_ids = fixture_values("component");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_component_id_dedup(component_type_ids.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_component_id_dedup(component_type_ids.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_UNSTABLE_COMPONENT_ID_DEDUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(
        optimized_component_id_dedup(component_type_ids),
        UNIQUE_ENTRY_COUNT
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
