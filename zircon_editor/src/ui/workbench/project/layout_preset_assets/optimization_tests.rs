use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::asset::AssetUri;

use super::list_layout_preset_assets;

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

fn fixture_names(prefix: &str) -> Vec<String> {
    (0..ENTRY_COUNT)
        .map(|index| format!("{prefix}.{:04}", index % UNIQUE_ENTRY_COUNT))
        .collect()
}

fn legacy_sort_dedup(mut names: Vec<String>) -> usize {
    names.sort();
    names.dedup();
    names.len()
}

fn optimized_sort_dedup(mut names: Vec<String>) -> usize {
    names.sort_unstable();
    names.dedup();
    names.len()
}

fn legacy_merge_dedup(asset_names: Vec<String>, config_names: Vec<String>) -> usize {
    let mut names = asset_names;
    names.extend(config_names);
    names.sort();
    names.dedup();
    names.len()
}

fn optimized_merge_dedup(asset_names: Vec<String>, config_names: Vec<String>) -> usize {
    let mut names = asset_names;
    names.extend(config_names);
    names.sort_unstable();
    names.dedup();
    names.len()
}

#[test]
fn editor13_unstable_preset_name_dedup_preserves_sorted_unique_projection() {
    let locators = (0..ENTRY_COUNT)
        .map(|index| {
            AssetUri::parse(&format!(
                "res://editor/layout-presets/preset-{}.workbench-layout.json",
                index % UNIQUE_ENTRY_COUNT
            ))
            .expect("fixture locator should parse")
        })
        .collect::<Vec<_>>();
    let names = list_layout_preset_assets(locators);

    assert_eq!(names.len(), UNIQUE_ENTRY_COUNT);
    assert_eq!(names.first().map(String::as_str), Some("preset-0"));
    assert_eq!(names.last().map(String::as_str), Some("preset-999"));
}

#[test]
fn editor13_unstable_preset_name_dedup_source_contract() {
    let assets_source = include_str!("../layout_preset_assets.rs");
    assert!(assets_source.contains("preset_names.sort_unstable();"));
    assert!(!assets_source.contains("preset_names.sort();"));

    let persistence_source = include_str!("../../../host/layout_persistence.rs");
    assert!(persistence_source.contains("names.sort_unstable();"));
    assert!(!persistence_source.contains("names.sort();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor13_unstable_preset_asset_name_dedup_bench() {
    let names = fixture_names("preset");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_sort_dedup(names.clone()));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_sort_dedup(names.clone()));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR13_UNSTABLE_PRESET_ASSET_NAME_DEDUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} unique_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        UNIQUE_ENTRY_COUNT,
    );
    assert_eq!(optimized_sort_dedup(names), UNIQUE_ENTRY_COUNT);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor13_unstable_preset_name_merge_dedup_bench() {
    let asset_names = fixture_names("asset");
    let config_names = fixture_names("config");
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_merge_dedup(
                asset_names.clone(),
                config_names.clone(),
            ));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_merge_dedup(
                asset_names.clone(),
                config_names.clone(),
            ));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR13_UNSTABLE_PRESET_NAME_MERGE_DEDUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} asset_entries={} config_entries={} merged_entries={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        ENTRY_COUNT,
        ENTRY_COUNT * 2,
    );
    assert_eq!(
        optimized_merge_dedup(asset_names, config_names),
        UNIQUE_ENTRY_COUNT * 2
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
