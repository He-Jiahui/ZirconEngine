use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{collect_assets, ZrPackMissingDependency, ZrPackTrimInputAsset, ZrPackTrimPlanner};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 256;
const ASSET_COUNT: usize = 2_048;
const DUPLICATE_COUNT: usize = ASSET_COUNT / 2;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_assets() -> Vec<ZrPackTrimInputAsset> {
    let mut assets = Vec::with_capacity(ASSET_COUNT + DUPLICATE_COUNT);
    for index in 0..ASSET_COUNT {
        assets.push(ZrPackTrimInputAsset::new(format!("asset.{index:04}")));
    }
    for index in (0..DUPLICATE_COUNT).rev() {
        assets.push(
            ZrPackTrimInputAsset::new(format!("asset.{index:04}")).with_label("duplicate-input"),
        );
    }
    assets
}

fn legacy_collect_assets(
    assets: impl IntoIterator<Item = ZrPackTrimInputAsset>,
) -> (
    BTreeMap<String, ZrPackTrimInputAsset>,
    Vec<String>,
    Vec<String>,
) {
    let mut asset_map = BTreeMap::new();
    let mut duplicate_assets = Vec::new();
    let mut diagnostics = Vec::new();
    for asset in assets {
        if asset_map.contains_key(&asset.path) {
            diagnostics.push(format!("asset {} is duplicated in trim input", asset.path));
            duplicate_assets.push(asset.path);
            continue;
        }
        asset_map.insert(asset.path.clone(), asset);
    }
    (asset_map, duplicate_assets, diagnostics)
}

fn report_sort_fixture() -> (Vec<ZrPackMissingDependency>, Vec<String>, Vec<String>) {
    let missing_dependencies = (0..ASSET_COUNT)
        .rev()
        .map(|index| {
            ZrPackMissingDependency::new(
                format!("asset.{:04}", index % (ASSET_COUNT / 2)),
                format!("missing.{:04}", index),
            )
        })
        .collect();
    let duplicate_assets = (0..ASSET_COUNT)
        .rev()
        .map(|index| format!("asset.{:04}", index % (ASSET_COUNT / 2)))
        .collect();
    let diagnostics = (0..ASSET_COUNT)
        .rev()
        .map(|index| format!("diagnostic.{:04}", index % (ASSET_COUNT / 2)))
        .collect();
    (missing_dependencies, duplicate_assets, diagnostics)
}

fn legacy_sort_report_vectors(
    missing_dependencies: &mut [ZrPackMissingDependency],
    duplicate_assets: &mut Vec<String>,
    diagnostics: &mut [String],
) {
    missing_dependencies.sort_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then(left.dependency.cmp(&right.dependency))
    });
    duplicate_assets.sort();
    duplicate_assets.dedup();
    diagnostics.sort();
}

fn optimized_sort_report_vectors(
    missing_dependencies: &mut [ZrPackMissingDependency],
    duplicate_assets: &mut Vec<String>,
    diagnostics: &mut [String],
) {
    missing_dependencies.sort_unstable_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then(left.dependency.cmp(&right.dependency))
    });
    duplicate_assets.sort_unstable();
    duplicate_assets.dedup();
    diagnostics.sort_unstable();
}

#[test]
fn runtime04_pack_trim_entry_lookup_preserves_first_asset_and_duplicates() {
    let assets = fixture_assets();
    let (optimized_map, optimized_duplicates, optimized_diagnostics) =
        collect_assets(assets.clone());
    let (legacy_map, legacy_duplicates, legacy_diagnostics) = legacy_collect_assets(assets);
    assert_eq!(optimized_map, legacy_map);
    assert_eq!(optimized_duplicates, legacy_duplicates);
    assert_eq!(optimized_diagnostics, legacy_diagnostics);
    assert_eq!(optimized_map.len(), ASSET_COUNT);
    assert_eq!(optimized_duplicates.len(), DUPLICATE_COUNT);
    assert!(optimized_map["asset.0000"].labels.is_empty());
}

#[test]
fn runtime04_pack_trim_report_sorting_preserves_ordered_contents() {
    let (mut missing_legacy, mut duplicates_legacy, mut diagnostics_legacy) = report_sort_fixture();
    let (mut missing_optimized, mut duplicates_optimized, mut diagnostics_optimized) =
        report_sort_fixture();
    legacy_sort_report_vectors(
        &mut missing_legacy,
        &mut duplicates_legacy,
        &mut diagnostics_legacy,
    );
    optimized_sort_report_vectors(
        &mut missing_optimized,
        &mut duplicates_optimized,
        &mut diagnostics_optimized,
    );
    assert_eq!(missing_optimized, missing_legacy);
    assert_eq!(duplicates_optimized, duplicates_legacy);
    assert_eq!(diagnostics_optimized, diagnostics_legacy);
}

#[test]
fn runtime04_pack_trim_source_contracts() {
    let source = include_str!("../trim.rs");
    let collect_source = source
        .split_once("fn collect_assets")
        .expect("collect_assets should exist")
        .1
        .split_once("fn reachable_asset_closure")
        .expect("reachable closure should follow collect_assets")
        .0;
    assert!(collect_source.contains("Entry::Occupied"));
    assert!(collect_source.contains("Entry::Vacant"));
    assert!(!collect_source.contains("asset_map.contains_key"));
    assert!(source.contains("missing_dependencies.sort_unstable_by"));
    assert!(source.contains("duplicate_assets.sort_unstable();"));
    assert!(source.contains("diagnostics.sort_unstable();"));
}

#[test]
fn runtime04_pack_trim_planner_keeps_deterministic_report_order() {
    let report = ZrPackTrimPlanner::trim(
        super::ZrPackTrimConfig::new(["asset.0000", "asset.missing"]),
        [
            ZrPackTrimInputAsset::new("asset.0000").with_dependency("asset.missing-dependency"),
            ZrPackTrimInputAsset::new("asset.0001"),
        ],
    );
    assert_eq!(report.included_assets, vec!["asset.0000".to_string()]);
    assert_eq!(report.missing_dependencies.len(), 2);
    assert!(report
        .diagnostics
        .windows(2)
        .all(|window| window[0] <= window[1]));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_trim_entry_lookup_bench() {
    let assets = fixture_assets();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_collect_assets(assets.clone()));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(collect_assets(assets.clone()));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME04_PACK_TRIM_ENTRY_LOOKUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} assets={} duplicates={} map_lookups=2->1",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ASSET_COUNT,
        DUPLICATE_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_trim_report_sorting_bench() {
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let (mut missing, mut duplicates, mut diagnostics) = report_sort_fixture();
                legacy_sort_report_vectors(&mut missing, &mut duplicates, &mut diagnostics);
                black_box((missing, duplicates, diagnostics));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let (mut missing, mut duplicates, mut diagnostics) = report_sort_fixture();
                optimized_sort_report_vectors(&mut missing, &mut duplicates, &mut diagnostics);
                black_box((missing, duplicates, diagnostics));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME04_PACK_TRIM_REPORT_SORTING_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} missing={} duplicate_input={} unique_duplicates={} stable_sorts=3->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ASSET_COUNT,
        ASSET_COUNT,
        DUPLICATE_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
