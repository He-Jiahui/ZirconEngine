use std::collections::{BTreeMap, BTreeSet, HashSet};
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

fn fixture_base_hashes() -> HashSet<[u8; 32]> {
    (0..ASSET_COUNT)
        .filter(|index| index % 2 == 0)
        .map(|index| [index as u8; 32])
        .collect()
}

fn fixture_target_paths(assets: &[ZrPackAssetEntry]) -> HashSet<&str> {
    assets
        .iter()
        .filter(|asset| asset.path.ends_with(".bin") && !asset.path.ends_with("4095.bin"))
        .map(|asset| asset.path.as_str())
        .collect()
}

fn legacy_removed_assets(
    base_assets: &[ZrPackAssetEntry],
    target_paths: &HashSet<&str>,
) -> Vec<String> {
    base_assets
        .iter()
        .filter(|asset| !target_paths.contains(asset.path.as_str()))
        .map(|asset| asset.path.clone())
        .collect()
}

fn legacy_delta_asset_changes(
    base_hashes: &HashSet<[u8; 32]>,
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
    let source = include_str!("../../delta.rs");
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

fn hash_for(index: usize) -> [u8; 32] {
    let mut hash = [0_u8; 32];
    hash[..8].copy_from_slice(&(index as u64).to_le_bytes());
    hash
}

fn ordered_membership_workload(paths: &[String], hashes: &[[u8; 32]]) -> usize {
    let path_index = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let hash_index = hashes.iter().copied().collect::<BTreeSet<_>>();
    paths
        .iter()
        .rev()
        .filter(|path| path_index.contains(path.as_str()))
        .count()
        + hashes
            .iter()
            .rev()
            .filter(|hash| hash_index.contains(*hash))
            .count()
}

fn hash_membership_workload(paths: &[String], hashes: &[[u8; 32]]) -> usize {
    let mut path_index = HashSet::with_capacity(paths.len());
    path_index.extend(paths.iter().map(String::as_str));
    let mut hash_index = HashSet::with_capacity(hashes.len());
    hash_index.extend(hashes.iter().copied());
    paths
        .iter()
        .rev()
        .filter(|path| path_index.contains(path.as_str()))
        .count()
        + hashes
            .iter()
            .rev()
            .filter(|hash| hash_index.contains(*hash))
            .count()
}

#[test]
fn optimization_batch_ia_runtime610_delta_membership_uses_preallocated_hash_indices() {
    let source = include_str!("../../delta.rs");
    let writer = source
        .split("impl ZrPackDeltaWriter")
        .nth(1)
        .expect("delta writer")
        .split("impl ZrPackDeltaReader")
        .next()
        .expect("bounded delta writer");
    let semantics = source
        .split("fn validate_delta_manifest_semantics")
        .nth(1)
        .expect("delta semantic validator")
        .split("fn read_delta_chunk_bytes")
        .next()
        .expect("bounded delta semantic validator");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashSet};"));
    assert!(writer.contains("HashSet::with_capacity(base.manifest().pack.chunks.len())"));
    assert!(writer.contains("HashSet::with_capacity(target.manifest().assets.len())"));
    assert!(semantics.contains("HashSet::with_capacity(manifest.target.assets.len())"));
    assert!(semantics.contains("HashSet::with_capacity(manifest.base.pack.chunks.len())"));
    assert!(semantics.contains("collect::<BTreeSet<_>>()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ia_runtime610_hash_delta_membership_performance_evidence() {
    const ITEMS: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;
    let path_suffix = "x".repeat(96);
    let paths = (0..ITEMS)
        .map(|index| format!("assets/shared-prefix-{path_suffix}-{index:05}.bin"))
        .collect::<Vec<_>>();
    let hashes = (0..ITEMS).map(hash_for).collect::<Vec<_>>();
    let measure_ordered = || {
        let started = Instant::now();
        black_box(ordered_membership_workload(
            black_box(&paths),
            black_box(&hashes),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_hash = || {
        let started = Instant::now();
        black_box(hash_membership_workload(
            black_box(&paths),
            black_box(&hashes),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_ordered());
        black_box(measure_hash());
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_ordered());
            hash_samples.push(measure_hash());
        } else {
            hash_samples.push(measure_hash());
            ordered_samples.push(measure_ordered());
        }
    }
    ordered_samples.sort_unstable();
    hash_samples.sort_unstable();
    let ordered_p50 = ordered_samples[8];
    let ordered_p95 = ordered_samples[16];
    let hash_p50 = hash_samples[8];
    let hash_p95 = hash_samples[16];
    println!(
        "RUNTIME610_HASH_DELTA_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 items={ITEMS} path_bytes=127 ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} membership_complexity=log_n->amortized_constant target_ratio_bp=5000"
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= ordered_p95.saturating_mul(5_000),
        "hash membership P95 {hash_p95} ns exceeded 50% of ordered membership {ordered_p95} ns"
    );
}
