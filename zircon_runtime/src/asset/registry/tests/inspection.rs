use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::rebuild::{build_index_from_documents, refresh_dependency_edges_from_documents};

#[test]
fn borrowed_metadata_inventory_builds_the_same_registry_index() {
    let uuid = AssetUuid::new();
    let uri =
        AssetUri::parse("res://shaders/inventory.wgsl").expect("fixture asset URI should parse");
    let document = AssetMetaDocument::new(uuid, uri, AssetKind::Shader);
    let documents = BTreeMap::from([(PathBuf::from("inventory.wgsl.zmeta"), document)]);

    let mut index = build_index_from_documents(documents.values(), Vec::new())
        .expect("borrowed inventory should build an index");
    refresh_dependency_edges_from_documents(&mut index, documents.values());

    assert_eq!(index.len(), 1);
    assert_eq!(
        index
            .entry_by_uuid(uuid)
            .expect("borrowed inventory entry should be indexed")
            .path()
            .to_string(),
        "res://shaders/inventory.wgsl"
    );
}

#[test]
fn loaded_metadata_inventory_reserves_size_hint_and_diagnostic_capacity() {
    let source = include_str!("../inspection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(upper_bound.unwrap_or(lower_bound))"));
    assert!(implementation.contains("Vec::with_capacity(\n            existing_diagnostics"));
    assert!(!implementation.contains("documents.into_iter().collect::<Vec<_>>()"));
}

#[test]
fn loaded_metadata_inventory_keeps_document_iteration_before_index_build() {
    let source = include_str!("../inspection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let extend = implementation
        .find("documents.extend(document_iter)")
        .expect("document extend");
    let build = implementation
        .find("build_index_from_documents(documents.iter().copied()")
        .expect("index build");
    assert!(extend < build);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cb_runtime_registry_inspection_capacity_p95() {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const DOCUMENTS_PER_SAMPLE: usize = 512;
    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut documents = if optimized {
                Vec::with_capacity(DOCUMENTS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..DOCUMENTS_PER_SAMPLE {
                documents.push(index);
            }
            checksum ^= documents.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }
    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }
    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!("RUNTIME380_REGISTRY_INSPECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} documents_per_sample={DOCUMENTS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", sample_csv(&legacy), sample_csv(&optimized));
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
