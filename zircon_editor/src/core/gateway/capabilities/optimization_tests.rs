use std::hint::black_box;
use std::time::Instant;

use super::{PluginActivationState, PluginSummaryEntry, RuntimeCapabilities, SessionProfileKind};

const CAPABILITY_COUNT: usize = 4_096;
const UNIQUE_CAPABILITIES: usize = 512;
const PLUGIN_COUNT: usize = 4_096;
const UNIQUE_PLUGINS: usize = 512;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 16;

fn capability_fixture() -> Vec<String> {
    (0..CAPABILITY_COUNT)
        .rev()
        .map(|index| format!("capability.{:04}", index % UNIQUE_CAPABILITIES))
        .collect()
}

fn plugin_fixture() -> Vec<PluginSummaryEntry> {
    (0..PLUGIN_COUNT)
        .rev()
        .map(|index| {
            let key = index % UNIQUE_PLUGINS;
            PluginSummaryEntry::new(
                format!("plugin.{key:04}"),
                format!("1.{}.0", key % 7),
                match key % 3 {
                    0 => PluginActivationState::Active,
                    1 => PluginActivationState::Disabled,
                    _ => PluginActivationState::Rejected,
                },
            )
        })
        .collect()
}

fn legacy_capabilities(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn legacy_plugins(mut values: Vec<PluginSummaryEntry>) -> Vec<PluginSummaryEntry> {
    values.sort_by(|left, right| {
        (&left.id, &left.version, left.activation).cmp(&(
            &right.id,
            &right.version,
            right.activation,
        ))
    });
    values.dedup();
    values
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn editor06_runtime_capabilities_unstable_sort_preserves_projection() {
    let expected_capabilities = legacy_capabilities(capability_fixture());
    let expected_plugins = legacy_plugins(plugin_fixture());
    let projection = RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        capability_fixture(),
        plugin_fixture(),
    );
    assert_eq!(projection.core_capabilities(), expected_capabilities);
    assert_eq!(projection.plugin_summary(), expected_plugins);
    assert_eq!(projection.core_capabilities().len(), UNIQUE_CAPABILITIES);
    assert_eq!(projection.plugin_summary().len(), UNIQUE_PLUGINS);
}

#[test]
fn editor06_runtime_capabilities_unstable_sort_source_contract() {
    let source = include_str!("../capabilities.rs");
    assert!(source.contains("core_capabilities.sort_unstable()"));
    assert!(source.contains("plugin_summary.sort_unstable_by"));
    assert!(!source.contains("core_capabilities.sort();"));
    assert!(!source.contains("plugin_summary.sort_by("));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_runtime_capabilities_unstable_sort_bench() {
    let legacy_capability_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_capabilities(capability_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_capability_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = capability_fixture();
                values.sort_unstable();
                values.dedup();
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_plugin_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_plugins(plugin_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_plugin_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = plugin_fixture();
                values.sort_unstable_by(|left, right| {
                    (&left.id, &left.version, left.activation).cmp(&(
                        &right.id,
                        &right.version,
                        right.activation,
                    ))
                });
                values.dedup();
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_capability_p95 = percentile_95(legacy_capability_samples);
    let optimized_capability_p95 = percentile_95(optimized_capability_samples);
    let legacy_plugin_p95 = percentile_95(legacy_plugin_samples);
    let optimized_plugin_p95 = percentile_95(optimized_plugin_samples);
    println!(
        "EDITOR06_RUNTIME_CAPABILITIES_UNSTABLE_SORT_BENCH_V1 capability_legacy_p95_ns={} capability_optimized_p95_ns={} plugin_legacy_p95_ns={} plugin_optimized_p95_ns={} samples={} iterations={} capabilities={} unique_capabilities={} plugins={} unique_plugins={} stable_sorts=2->0",
        legacy_capability_p95,
        optimized_capability_p95,
        legacy_plugin_p95,
        optimized_plugin_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        CAPABILITY_COUNT,
        UNIQUE_CAPABILITIES,
        PLUGIN_COUNT,
        UNIQUE_PLUGINS,
    );
    assert!(
        optimized_capability_p95.saturating_mul(100) <= legacy_capability_p95.saturating_mul(95),
        "optimized capability sort p95 should be at most 95% of legacy p95"
    );
    assert!(
        optimized_plugin_p95.saturating_mul(100) <= legacy_plugin_p95.saturating_mul(95),
        "optimized plugin sort p95 should be at most 95% of legacy p95"
    );
}
