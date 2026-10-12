use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent};
use zircon_runtime::asset::{AssetId, AssetKind, AssetUri, AssetUuid};

use super::{asset_watch_events, dirty_catalog_records};
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetCatalogGeneration, EditorAssetCatalogRecord,
};

#[test]
fn incomplete_runtime_rename_is_a_safe_added_event() {
    let uri = AssetUri::parse("res://models/ship.glb").unwrap();
    assert_eq!(
        asset_watch_events(&[AssetChange::new(
            AssetChangeKind::Renamed,
            uri.clone(),
            None
        )]),
        vec![AssetWatchEvent::Added(uri)]
    );
}

#[test]
fn repeated_watch_events_stage_the_same_dirty_catalog_row_once() {
    let (catalog, uri, uuid) = clean_watch_catalog(0, 0);
    let once = dirty_catalog_records(&catalog, &[AssetWatchEvent::Modified(uri.clone())]);
    let repeated = dirty_catalog_records(
        &catalog,
        &[
            AssetWatchEvent::Modified(uri.clone()),
            AssetWatchEvent::Modified(uri.clone()),
            AssetWatchEvent::Renamed {
                from: uri.clone(),
                to: uri,
            },
        ],
    );

    assert_eq!(repeated, once);
    assert_eq!(repeated.len(), 1);
    assert!(repeated.values().next().unwrap().dirty);
    assert!(!catalog.catalog_record(&uuid.to_string()).unwrap().dirty);
}

#[test]
#[ignore = "Windows Release watcher projection evidence"]
fn editor04_watch_storm_dirty_stage_release_benchmark() {
    const EVENT_COUNT: usize = 4_096;
    const DIAGNOSTIC_COUNT: usize = 64;
    const DIAGNOSTIC_BYTES: usize = 256;
    const SAMPLE_COUNT: usize = 21;
    let (catalog, uri, _) = clean_watch_catalog(DIAGNOSTIC_COUNT, DIAGNOSTIC_BYTES);
    let events = vec![AssetWatchEvent::Modified(uri); EVENT_COUNT];
    assert_eq!(
        legacy_dirty_catalog_records(&catalog, &events),
        dirty_catalog_records(&catalog, &events)
    );
    black_box(legacy_dirty_catalog_records(&catalog, &events));
    black_box(dirty_catalog_records(&catalog, &events));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_dirty_stage(|| {
                legacy_dirty_catalog_records(&catalog, &events)
            }));
            optimized_samples.push(benchmark_dirty_stage(|| {
                dirty_catalog_records(&catalog, &events)
            }));
        } else {
            optimized_samples.push(benchmark_dirty_stage(|| {
                dirty_catalog_records(&catalog, &events)
            }));
            legacy_samples.push(benchmark_dirty_stage(|| {
                legacy_dirty_catalog_records(&catalog, &events)
            }));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let legacy_p99 = percentile(&legacy_samples, 99);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let optimized_p99 = percentile(&optimized_samples, 99);
    println!(
        "PERF_RESULT EDITOR04_WATCH_STORM_DIRTY_STAGE_BENCH_V1 events={EVENT_COUNT} unique_assets=1 diagnostics={DIAGNOSTIC_COUNT} diagnostic_bytes={DIAGNOSTIC_BYTES} samples={SAMPLE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99} threshold_p95_ratio=0.70"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "unique dirty stage p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn clean_watch_catalog(
    diagnostic_count: usize,
    diagnostic_bytes: usize,
) -> (EditorAssetCatalogGeneration, AssetUri, AssetUuid) {
    let uri = AssetUri::parse("res://textures/sky.png").unwrap();
    let uuid = AssetUuid::from_stable_label("sky");
    let id = AssetId::from_asset_uuid(uuid);
    let diagnostics = (0..diagnostic_count)
        .map(|index| format!("{index:04}-{}", "d".repeat(diagnostic_bytes)))
        .collect::<Vec<_>>();
    let public = EditorAssetCatalogRecord {
        uuid: uuid.to_string(),
        id: id.to_string(),
        locator: uri.to_string(),
        kind: AssetKind::Texture,
        display_name: "sky".to_string(),
        file_name: "sky.png".to_string(),
        extension: "png".to_string(),
        preview_state: PreviewState::Ready,
        meta_path: String::new(),
        preview_artifact_path: String::new(),
        source_mtime_unix_ms: 0,
        source_hash: String::new(),
        dirty: false,
        diagnostics: diagnostics.clone(),
        direct_reference_uuids: Vec::new(),
    };
    let catalog_record = AssetCatalogRecord {
        asset_uuid: uuid,
        asset_id: id,
        locator: uri.clone(),
        kind: AssetKind::Texture,
        display_name: "sky".to_string(),
        file_name: "sky.png".to_string(),
        extension: "png".to_string(),
        meta_path: Default::default(),
        meta: AssetMetaDocument::new(uuid, uri.clone(), AssetKind::Texture),
        source_mtime_unix_ms: 0,
        source_hash: String::new(),
        preview_state: PreviewState::Ready,
        preview_artifact_path: Default::default(),
        dirty: false,
        diagnostics,
        direct_references: Vec::new(),
    };
    let catalog = EditorAssetCatalogGeneration::from_parts(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        1,
        1,
        Vec::new(),
        vec![Arc::new(public)],
        vec![None],
        vec![Some(Arc::new(catalog_record))],
    );
    (catalog, uri, uuid)
}

fn legacy_dirty_catalog_records(
    catalog: &EditorAssetCatalogGeneration,
    events: &[AssetWatchEvent],
) -> BTreeMap<String, AssetCatalogRecord> {
    let mut updates = BTreeMap::new();
    for event in events {
        let AssetWatchEvent::Modified(uri) = event else {
            panic!("legacy benchmark models a modify storm");
        };
        let Some(asset) = catalog.asset_by_locator(&uri.to_string()) else {
            continue;
        };
        let Some(current) = catalog.catalog_record(&asset.uuid) else {
            continue;
        };
        if current.dirty {
            continue;
        }
        let mut updated = (*current).clone();
        updated.dirty = true;
        updates.insert(updated.asset_uuid.to_string(), updated);
    }
    updates
}

fn benchmark_dirty_stage(
    mut operation: impl FnMut() -> BTreeMap<String, AssetCatalogRecord>,
) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
