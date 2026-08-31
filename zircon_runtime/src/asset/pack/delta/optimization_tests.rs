use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use super::super::ZrPackAssetEntry;
use super::{collect_delta_asset_changes, collect_removed_assets};

const ASSET_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;

fn fixture_assets() -> Vec<ZrPackAssetEntry> {
    (0..ASSET_COUNT)
        .map(|index| ZrPackAssetEntry::new(format!("assets/{index:05}.bin"), [index as u8; 32], 16))
        .collect()
}

fn fixture_base_hashes() -> BTreeSet<[u8; 32]> {
    (0..ASSET_COUNT)
        .filter(|index| index % 2 == 0)
        .map(|index| [index as u8; 32])
        .collect()
}

fn fixture_target_paths(assets: &[ZrPackAssetEntry]) -> BTreeSet<&str> {
    assets
        .iter()
        .filter(|asset| asset.path.ends_with(".bin") && !asset.path.ends_with("4095.bin"))
        .map(|asset| asset.path.as_str())
        .collect()
}

fn legacy_removed_assets(
    base_assets: &[ZrPackAssetEntry],
    target_paths: &BTreeSet<&str>,
) -> Vec<String> {
    base_assets
        .iter()
        .filter(|asset| !target_paths.contains(asset.path.as_str()))
        .map(|asset| asset.path.clone())
        .collect()
}

fn legacy_delta_asset_changes(
    base_hashes: &BTreeSet<[u8; 32]>,
    target_assets: &[ZrPackAssetEntry],
) -> (
    Vec<ZrPackAssetEntry>,
    Vec<String>,
    Vec<String>,
    BTreeMap<[u8; 32], String>,
) {
    let mut changed_asset_entries = Vec::new();
    let mut changed_assets = Vec::new();
    let mut reused_assets = Vec::new();
    let mut chunk_source_paths = BTreeMap::new();
    for asset in target_assets {
        if base_hashes.contains(&asset.chunk_hash) {
            reused_assets.push(asset.path.clone());
            continue;
        }
        chunk_source_paths
            .entry(asset.chunk_hash)
            .or_insert_with(|| asset.path.clone());
        changed_assets.push(asset.path.clone());
        changed_asset_entries.push(asset.clone());
    }
    (
        changed_asset_entries,
        changed_assets,
        reused_assets,
        chunk_source_paths,
    )
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_pack_delta_capacity_preserves_change_and_removal_sets() {
    let assets = fixture_assets();
    let base_hashes = fixture_base_hashes();
    let target_paths = fixture_target_paths(&assets);

    assert_eq!(
        collect_removed_assets(&assets, &target_paths),
        legacy_removed_assets(&assets, &target_paths)
    );
    assert_eq!(
        collect_delta_asset_changes(&base_hashes, &assets),
        legacy_delta_asset_changes(&base_hashes, &assets)
    );
}

#[test]
fn runtime04_pack_delta_capacity_source_contract() {
    let source = include_str!("../delta.rs");
    assert!(source.contains("Vec::with_capacity(base_assets.len())"));
    assert!(source.contains("Vec::with_capacity(target_assets.len())"));
    assert!(source.contains("Vec::with_capacity(chunk_source_paths.len())"));
    assert!(!source.contains("let mut changed_asset_entries = Vec::new();"));
    assert!(!source.contains("let mut reused_assets = Vec::new();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_delta_capacity_bench() {
    let assets = fixture_assets();
    let base_hashes = fixture_base_hashes();
    let target_paths = fixture_target_paths(&assets);
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_removed_assets(&assets, &target_paths));
                black_box(legacy_delta_asset_changes(&base_hashes, &assets));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(collect_removed_assets(&assets, &target_paths));
                black_box(collect_delta_asset_changes(&base_hashes, &assets));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_PACK_DELTA_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} assets={} reserved_removed_slots=0->{} reserved_change_slots=0->{}",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        ASSET_COUNT,
        ASSET_COUNT,
        ASSET_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}
