use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::asset_registry_index::source_locator;
use super::super::{AssetRegistryEntry, AssetRegistryIndex};
use super::dependency_paths;

const ENTRY_COUNT: usize = 4_096;
const DEPENDENCY_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;

fn fixture_index() -> (AssetRegistryIndex, AssetUri) {
    let source = AssetUri::parse("res://models/capacity.glb").expect("valid source URI");
    let entries = (0..ENTRY_COUNT).map(|index| {
        AssetRegistryEntry::new(
            AssetUuid::from_stable_label(&format!("runtime04-source-entry-{index}")),
            AssetUri::parse(&format!("res://models/capacity.glb#Mesh{index}"))
                .expect("valid labeled URI"),
            AssetKind::Data,
            "capacity-digest",
        )
    });
    (
        AssetRegistryIndex::from_entries(entries).expect("fixture index should build"),
        source,
    )
}

fn legacy_source_entries(
    index: &AssetRegistryIndex,
    locator: &AssetUri,
) -> Vec<AssetRegistryEntry> {
    index
        .entry_uuids_by_source
        .get(&source_locator(locator))
        .into_iter()
        .flatten()
        .filter_map(|uuid| index.entries_by_uuid.get(uuid))
        .cloned()
        .collect()
}

fn fixture_document() -> AssetMetaDocument {
    let mut document = AssetMetaDocument::new(
        AssetUuid::from_stable_label("runtime04-targeted-capacity-owner"),
        AssetUri::parse("res://models/dependencies.glb").expect("valid fixture URI"),
        AssetKind::Data,
    );
    document.entries = (0..DEPENDENCY_COUNT)
        .map(|index| AssetMetaEntry {
            uuid: AssetUuid::from_stable_label(&format!("runtime04-targeted-dependency-{index}")),
            url: AssetUri::parse(&format!("res://models/dependencies.glb#Mesh{index}"))
                .expect("valid labeled fixture URI"),
            asset_kind: AssetKind::Data,
            artifact_locator: None,
            dependencies: vec![
                AssetUri::parse(&format!("res://dependencies/{index}")).expect("valid dependency")
            ],
            tags: Default::default(),
        })
        .collect();
    document
}

fn legacy_dependency_paths(meta: &AssetMetaDocument) -> Vec<(AssetUuid, Vec<AssetUri>)> {
    let mut dependencies = Vec::new();
    if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
        dependencies.push((meta.uuid, meta.dependencies.clone()));
    }
    dependencies.extend(
        meta.entries
            .iter()
            .map(|entry| (entry.uuid, entry.dependencies.clone())),
    );
    dependencies
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_targeted_capacity_preserves_source_entries_and_dependency_paths() {
    let (index, source) = fixture_index();
    let legacy_entries = legacy_source_entries(&index, &source);
    let optimized_entries = index.source_entries(&source);
    let document = fixture_document();

    assert_eq!(optimized_entries.len(), ENTRY_COUNT);
    let mut legacy_paths = legacy_entries
        .iter()
        .map(|entry| entry.path().clone())
        .collect::<HashSet<_>>();
    let optimized_paths = optimized_entries
        .iter()
        .map(|entry| entry.path().clone())
        .collect::<HashSet<_>>();
    assert_eq!(legacy_paths, optimized_paths);
    legacy_paths.clear();
    assert_eq!(
        dependency_paths(&document),
        legacy_dependency_paths(&document)
    );
}

#[test]
fn runtime04_targeted_capacity_source_contract() {
    let source = include_str!("../targeted.rs");
    assert!(source.contains("Vec::with_capacity(uuids.map_or(0, HashSet::len))"));
    assert!(source
        .contains("Vec::with_capacity(meta.entries.len() + usize::from(has_root_dependencies))"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_targeted_capacity_bench() {
    let (index, source) = fixture_index();
    let document = fixture_document();
    let legacy_source_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_source_entries(&index, &source));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_source_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(index.source_entries(&source));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_dependency_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_dependency_paths(&document));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_dependency_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(dependency_paths(&document));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_source_p95 = percentile_95(legacy_source_samples);
    let optimized_source_p95 = percentile_95(optimized_source_samples);
    let legacy_dependency_p95 = percentile_95(legacy_dependency_samples);
    let optimized_dependency_p95 = percentile_95(optimized_dependency_samples);
    println!(
        "RUNTIME04_TARGETED_CAPACITY_BENCH_V1 source_legacy_p95_ns={} source_optimized_p95_ns={} dependency_legacy_p95_ns={} dependency_optimized_p95_ns={} samples={} iterations={} source_entries={} dependencies={} reserved_source_slots=0->{} reserved_dependency_slots=0->{}",
        legacy_source_p95,
        optimized_source_p95,
        legacy_dependency_p95,
        optimized_dependency_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        DEPENDENCY_COUNT,
        ENTRY_COUNT,
        DEPENDENCY_COUNT,
    );
    assert!(
        optimized_source_p95.saturating_mul(100) <= legacy_source_p95.saturating_mul(95),
        "optimized source-entry p95 should be at most 95% of legacy p95"
    );
    assert!(
        optimized_dependency_p95.saturating_mul(100) <= legacy_dependency_p95.saturating_mul(95),
        "optimized dependency-path p95 should be at most 95% of legacy p95"
    );
}
