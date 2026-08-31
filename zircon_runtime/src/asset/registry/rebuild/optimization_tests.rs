use std::hint::black_box;
use std::time::Instant;

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::AssetRegistryEntry;
use super::registry_entries;

const ENTRY_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;

fn fixture_document(entry_count: usize) -> AssetMetaDocument {
    let mut document = AssetMetaDocument::new(
        AssetUuid::from_stable_label("runtime04-registry-capacity-root"),
        AssetUri::parse("res://registry/capacity.data").expect("valid fixture URI"),
        AssetKind::Data,
    );
    document.source_digest = "capacity-digest".to_string();
    document.entries = (0..entry_count)
        .map(|index| AssetMetaEntry {
            uuid: AssetUuid::from_stable_label(&format!("runtime04-registry-capacity-{index}")),
            url: AssetUri::parse(&format!("res://registry/capacity.data#Entry{index}"))
                .expect("valid labeled fixture URI"),
            asset_kind: AssetKind::Data,
            artifact_locator: None,
            dependencies: Vec::new(),
            tags: Default::default(),
        })
        .collect();
    document
}

fn legacy_registry_entries(meta: &AssetMetaDocument) -> Vec<AssetRegistryEntry> {
    let mut entries = Vec::new();
    if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
        entries.push(
            AssetRegistryEntry::new(
                meta.uuid,
                meta.url.clone(),
                meta.asset_kind,
                meta.source_digest.clone(),
            )
            .with_tags(meta.tags.clone()),
        );
    }
    entries.extend(meta.entries.iter().map(|entry| {
        let tags = if entry.url.label().is_none() {
            meta.tags.clone()
        } else {
            entry.tags.clone()
        };
        AssetRegistryEntry::new(
            entry.uuid,
            entry.url.clone(),
            entry.asset_kind,
            meta.source_digest.clone(),
        )
        .with_tags(tags)
    }));
    entries
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_registry_capacity_projection_preserves_entries() {
    let document = fixture_document(128);
    let legacy = legacy_registry_entries(&document);
    let optimized = registry_entries(&document);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), 129);
}

#[test]
fn runtime04_registry_capacity_source_contract() {
    let source = include_str!("../rebuild.rs");
    assert!(source.contains("Vec::with_capacity(meta_paths.len())"));
    assert!(source.contains("HashMap::with_capacity(index.entries_by_uuid.len())"));
    assert!(source.contains("Vec::with_capacity(paths.len())"));
    assert!(source.contains("Vec::with_capacity(meta.entries.len() + usize::from(has_root_entry))"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_registry_capacity_projection_bench() {
    let document = fixture_document(ENTRY_COUNT);
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_registry_entries(&document));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(registry_entries(&document));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_REGISTRY_CAPACITY_PROJECTION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} reserved_slots=0->{}",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        ENTRY_COUNT + 1,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}
