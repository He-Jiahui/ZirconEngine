use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::registry::AssetRegistryFilter;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 64;
const ENTRY_COUNT: usize = 4_096;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_index() -> AssetRegistryIndex {
    let entries = (0..ENTRY_COUNT).rev().map(|index| {
        AssetRegistryEntry::new(
            AssetUuid::from_stable_label(&format!("registry-entry-{index:05}")),
            AssetUri::parse(&format!("res://registry/asset-{index:05}.zmeta"))
                .expect("valid registry fixture URI"),
            AssetKind::Data,
            format!("digest-{index:05}"),
        )
    });
    AssetRegistryIndex::from_entries(entries).expect("fixture entries have unique identity")
}

fn legacy_entries(index: &AssetRegistryIndex) -> Vec<&AssetRegistryEntry> {
    let mut entries = index.entries_by_uuid.values().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    entries
}

#[test]
fn runtime04_registry_records_asset_entry_sort_preserves_canonical_path_order() {
    let index = fixture_index();
    let legacy = legacy_entries(&index);
    let optimized = index.entries();

    assert_eq!(optimized, legacy);
    assert_eq!(index.get_assets(&AssetRegistryFilter::default()), legacy);
    assert_eq!(optimized.len(), ENTRY_COUNT);
    assert!(optimized
        .windows(2)
        .all(|window| window[0].path() <= window[1].path()));
}

#[test]
fn runtime04_registry_records_borrow_canonical_entries_without_sorting() {
    let index = fixture_index();
    let legacy = legacy_entries(&index);
    let borrowed = index.entries_iter().collect::<Vec<_>>();

    assert_eq!(borrowed, legacy);
    assert_eq!(borrowed, index.entries());
}

#[test]
fn runtime04_registry_records_keep_canonical_order_after_source_removal() {
    let mut index = fixture_index();
    let removed = AssetUri::parse("res://registry/asset-02048.zmeta").unwrap();

    index.remove_source_path(&removed);

    let paths = index
        .entries_iter()
        .map(|entry| entry.path().clone())
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), ENTRY_COUNT - 1);
    assert!(!paths.contains(&removed));
    assert_eq!(
        index.get_assets(&AssetRegistryFilter::default()),
        legacy_entries(&index)
    );
    assert!(paths.windows(2).all(|window| window[0] <= window[1]));
}

#[test]
fn runtime04_registry_records_keep_canonical_order_after_remove_and_reinsert() {
    let mut index = fixture_index();
    let path = AssetUri::parse("res://registry/asset-02048.zmeta").unwrap();
    let uuid = AssetUuid::from_stable_label("registry-entry-02048");

    index.remove_source_path(&path);
    index
        .insert_checked(AssetRegistryEntry::new(
            uuid,
            path.clone(),
            AssetKind::Data,
            "digest-02048",
        ))
        .expect("removed registry entry can be reinserted");

    let paths = index
        .entries_iter()
        .map(|entry| entry.path().clone())
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), ENTRY_COUNT);
    assert_eq!(
        index.get_assets(&AssetRegistryFilter::default()),
        legacy_entries(&index)
    );
    assert!(paths.windows(2).all(|window| window[0] <= window[1]));
    assert_eq!(
        index.entry_by_path(&path).map(AssetRegistryEntry::uuid),
        Some(uuid)
    );
}

#[test]
fn runtime04_registry_records_asset_entry_sort_source_contract() {
    let source = include_str!("../../asset_registry_index.rs");
    assert!(source.contains("uuids_by_canonical_path: BTreeMap<AssetUri, AssetUuid>"));
    assert!(source.contains("pub fn entries_iter("));
    assert!(!source.contains("entries.sort_unstable_by(|left, right|"));
    assert!(!source.contains("entries.sort_by(|left, right|"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_registry_records_asset_entry_sort_bench() {
    let index = fixture_index();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_entries(&index));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(index.entries());
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME04_ASSET_REGISTRY_CANONICAL_ITERATION_BENCH_V2 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} sorts_per_read=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 50,
        "canonical-index p95 should be at most 50% of copy-and-sort p95"
    );
}
