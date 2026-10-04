use std::collections::HashMap;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::{AssetId, AssetKind, AssetUri, AssetUuid};

use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetCatalogGeneration, EditorAssetCatalogRecord, PreviewScheduler,
};

use super::{
    merge_current_preview_results, preview_scheduler_for, snapshot_matching_preview_generation,
};

#[test]
fn editor04_preview_merge_keeps_matching_results_and_preserves_new_source_demand() {
    let clean_uuid = uuid(0);
    let stale_uuid = uuid(1);
    let mut current_clean = record(0, "stable", PreviewState::Ready, false);
    current_clean.preview_artifact_path = PathBuf::from("cache/stable.png");
    let current_stale = record(1, "old", PreviewState::Ready, false);
    let current = generation(vec![current_clean.clone(), current_stale]);

    let mut pending_clean = record(0, "stable", PreviewState::Dirty, true);
    pending_clean.preview_artifact_path = PathBuf::from("cache/pending.png");
    let pending_stale = record(1, "new", PreviewState::Dirty, true);
    let mut pending = HashMap::from([(clean_uuid, pending_clean), (stale_uuid, pending_stale)]);

    merge_current_preview_results(&mut pending, &current);
    let mut scheduler = preview_scheduler_for(&pending);

    let clean = &pending[&clean_uuid];
    assert_eq!(clean.preview_state, PreviewState::Ready);
    assert_eq!(clean.meta.preview_state, PreviewState::Ready);
    assert_eq!(
        clean.preview_artifact_path,
        current_clean.preview_artifact_path
    );
    assert!(!clean.dirty);
    assert!(scheduler.request_refresh(clean_uuid, true).is_none());

    let stale = &pending[&stale_uuid];
    assert_eq!(stale.preview_state, PreviewState::Dirty);
    assert_eq!(stale.meta.preview_state, PreviewState::Dirty);
    assert!(stale.dirty);
    assert!(scheduler.request_refresh(stale_uuid, true).is_some());
}

#[test]
fn editor04_preview_snapshot_rebases_a_later_preview_publication() {
    let asset_uuid = uuid(2);
    let expected = Arc::new(generation(vec![record(
        2,
        "stable",
        PreviewState::Dirty,
        true,
    )]));
    let state = RwLock::new(Arc::clone(&expected));
    let snapshot = {
        let current = state.read().expect("editor catalog read lock");
        snapshot_matching_preview_generation(&current, &expected)
            .expect("same generation must be captured")
    };

    let mut ready = record(2, "stable", PreviewState::Ready, false);
    ready.preview_artifact_path = PathBuf::from("cache/latest.png");
    let latest = Arc::new(
        expected
            .updated_catalog_record(ready.clone(), 2)
            .expect("preview completion publishes a new generation"),
    );
    *state.write().expect("editor catalog write lock") = Arc::clone(&latest);

    let mut pending = HashMap::from([(asset_uuid, record(2, "stable", PreviewState::Dirty, true))]);
    merge_current_preview_results(&mut pending, &snapshot);
    assert!(pending[&asset_uuid].dirty);
    assert_eq!(latest.catalog_revision, expected.catalog_revision);
    assert_eq!(latest.publish_epoch, 2);
    assert!(snapshot_matching_preview_generation(&latest, &expected).is_none());

    // The same-revision rebase uses the latest published preview, not the old snapshot.
    merge_current_preview_results(&mut pending, &latest);
    let mut scheduler = preview_scheduler_for(&pending);
    assert_eq!(pending[&asset_uuid].preview_state, PreviewState::Ready);
    assert_eq!(
        pending[&asset_uuid].preview_artifact_path,
        ready.preview_artifact_path
    );
    assert!(!pending[&asset_uuid].dirty);
    assert!(scheduler.request_refresh(asset_uuid, true).is_none());
}

#[test]
#[ignore = "Windows Release Editor04 catalog prepare evidence"]
fn editor04_preview_scheduler_single_scan_large_catalog_release_benchmark() {
    const WARMUP_COUNT: usize = 5;
    const SAMPLE_COUNT: usize = 31;
    for asset_count in [10_000, 100_000] {
        let records = (0..asset_count)
            .map(|index| record(index, "stable", PreviewState::Dirty, true))
            .collect::<Vec<_>>();
        let expected = Arc::new(generation(records.clone()));
        let state = RwLock::new(Arc::clone(&expected));
        let mut pending = records
            .into_iter()
            .map(|record| (record.asset_uuid, record))
            .collect::<HashMap<_, _>>();
        assert_eq!(pending.len(), asset_count);

        for warmup in 0..WARMUP_COUNT {
            if warmup % 2 == 0 {
                black_box(legacy_preview_prepare(&mut pending, &state, &expected));
                black_box(optimized_preview_prepare(&mut pending, &state, &expected));
            } else {
                black_box(optimized_preview_prepare(&mut pending, &state, &expected));
                black_box(legacy_preview_prepare(&mut pending, &state, &expected));
            }
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(|| {
                    legacy_preview_prepare(&mut pending, &state, &expected)
                }));
                optimized_samples.push(measure(|| {
                    optimized_preview_prepare(&mut pending, &state, &expected)
                }));
            } else {
                optimized_samples.push(measure(|| {
                    optimized_preview_prepare(&mut pending, &state, &expected)
                }));
                legacy_samples.push(measure(|| {
                    legacy_preview_prepare(&mut pending, &state, &expected)
                }));
            }
        }

        let legacy_stage = legacy_samples
            .iter()
            .map(|sample| sample.0)
            .collect::<Vec<_>>();
        let optimized_stage = optimized_samples
            .iter()
            .map(|sample| sample.0)
            .collect::<Vec<_>>();
        let legacy_lock = legacy_samples
            .iter()
            .map(|sample| sample.1)
            .collect::<Vec<_>>();
        let optimized_lock = optimized_samples
            .iter()
            .map(|sample| sample.1)
            .collect::<Vec<_>>();
        let legacy_p50 = percentile(&legacy_stage, 50);
        let legacy_p95 = percentile(&legacy_stage, 95);
        let legacy_p99 = percentile(&legacy_stage, 99);
        let optimized_p50 = percentile(&optimized_stage, 50);
        let optimized_p95 = percentile(&optimized_stage, 95);
        let optimized_p99 = percentile(&optimized_stage, 99);
        let legacy_lock_p50 = percentile(&legacy_lock, 50);
        let legacy_lock_p95 = percentile(&legacy_lock, 95);
        let legacy_lock_p99 = percentile(&legacy_lock, 99);
        let optimized_lock_p50 = percentile(&optimized_lock, 50);
        let optimized_lock_p95 = percentile(&optimized_lock, 95);
        let optimized_lock_p99 = percentile(&optimized_lock, 99);
        let p95_limit = if asset_count == 100_000 { 95 } else { 110 };
        println!(
            "PERF_RESULT EDITOR04_PREVIEW_SCHEDULER_LARGE_CATALOG_BENCH_V1 assets={asset_count} warmups={WARMUP_COUNT} samples={SAMPLE_COUNT} order=legacy_first_even_sample legacy_prepare_p50_ns={legacy_p50} legacy_prepare_p95_ns={legacy_p95} legacy_prepare_p99_ns={legacy_p99} optimized_prepare_p50_ns={optimized_p50} optimized_prepare_p95_ns={optimized_p95} optimized_prepare_p99_ns={optimized_p99} legacy_read_lock_p50_ns={legacy_lock_p50} legacy_read_lock_p95_ns={legacy_lock_p95} legacy_read_lock_p99_ns={legacy_lock_p99} optimized_read_lock_p50_ns={optimized_lock_p50} optimized_read_lock_p95_ns={optimized_lock_p95} optimized_read_lock_p99_ns={optimized_lock_p99} raw_legacy_prepare_ns={legacy_stage:?} raw_optimized_prepare_ns={optimized_stage:?} raw_legacy_read_lock_ns={legacy_lock:?} raw_optimized_read_lock_ns={optimized_lock:?} threshold_prepare_p95_ratio={p95_limit}% threshold_read_lock_p95_ratio=10% os={} arch={} package_version={}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            env!("CARGO_PKG_VERSION")
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(p95_limit),
            "preview prepare p95 {optimized_p95}ns exceeded {p95_limit}% of legacy {legacy_p95}ns at {asset_count} assets"
        );
        assert!(
            optimized_lock_p95.saturating_mul(100) <= legacy_lock_p95.saturating_mul(10),
            "read-lock hold p95 {optimized_lock_p95}ns exceeded 10% of legacy {legacy_lock_p95}ns at {asset_count} assets"
        );
    }
}

fn legacy_preview_prepare(
    pending: &mut HashMap<AssetUuid, AssetCatalogRecord>,
    state: &RwLock<Arc<EditorAssetCatalogGeneration>>,
    expected: &Arc<EditorAssetCatalogGeneration>,
) -> (PreviewScheduler, u128) {
    let mut scheduler = preview_scheduler_for(pending);
    black_box(&scheduler);
    let current = state.read().expect("editor catalog read lock");
    let lock_started = Instant::now();
    if Arc::ptr_eq(&current, expected) {
        merge_current_preview_results(pending, &current);
        scheduler = preview_scheduler_for(pending);
    }
    drop(current);
    (scheduler, lock_started.elapsed().as_nanos().max(1))
}

fn optimized_preview_prepare(
    pending: &mut HashMap<AssetUuid, AssetCatalogRecord>,
    state: &RwLock<Arc<EditorAssetCatalogGeneration>>,
    expected: &Arc<EditorAssetCatalogGeneration>,
) -> (PreviewScheduler, u128) {
    let (current, lock_duration) = {
        let current = state.read().expect("editor catalog read lock");
        let lock_started = Instant::now();
        let snapshot = snapshot_matching_preview_generation(&current, expected);
        drop(current);
        (snapshot, lock_started.elapsed().as_nanos().max(1))
    };
    if let Some(current) = current {
        merge_current_preview_results(pending, &current);
    }
    (preview_scheduler_for(pending), lock_duration)
}

fn measure(mut operation: impl FnMut() -> (PreviewScheduler, u128)) -> (u128, u128) {
    let started = Instant::now();
    let (scheduler, lock_duration) = operation();
    black_box(scheduler);
    (started.elapsed().as_nanos().max(1), lock_duration)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}

fn uuid(index: usize) -> AssetUuid {
    AssetUuid::from_stable_label(&format!("editor04-preview-{index:06}"))
}

fn record(
    index: usize,
    hash: &str,
    preview_state: PreviewState,
    dirty: bool,
) -> AssetCatalogRecord {
    let asset_uuid = uuid(index);
    let locator = AssetUri::parse(&format!("res://assets/asset_{index:06}.png"))
        .expect("valid catalog asset locator");
    let mut meta = AssetMetaDocument::new(asset_uuid, locator.clone(), AssetKind::Texture);
    meta.preview_state = preview_state;
    AssetCatalogRecord {
        asset_uuid,
        asset_id: AssetId::from_asset_uuid(asset_uuid),
        locator,
        kind: AssetKind::Texture,
        display_name: format!("Asset {index:06}"),
        file_name: format!("asset_{index:06}.png"),
        extension: "png".to_owned(),
        meta_path: PathBuf::from(format!("meta/asset_{index:06}.zmeta")),
        meta,
        source_mtime_unix_ms: 0,
        source_hash: hash.to_owned(),
        preview_state,
        preview_artifact_path: PathBuf::new(),
        dirty,
        diagnostics: Vec::new(),
        direct_references: Vec::new(),
    }
}

fn generation(records: Vec<AssetCatalogRecord>) -> EditorAssetCatalogGeneration {
    let assets = records.iter().map(public_record).map(Arc::new).collect();
    let details = vec![None; records.len()];
    let catalog_records = records
        .into_iter()
        .map(|record| Some(Arc::new(record)))
        .collect();
    EditorAssetCatalogGeneration::from_parts(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        1,
        1,
        Vec::new(),
        assets,
        details,
        catalog_records,
    )
}

fn public_record(record: &AssetCatalogRecord) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: record.asset_uuid.to_string(),
        id: record.asset_id.to_string(),
        locator: record.locator.to_string(),
        kind: record.kind,
        display_name: record.display_name.clone(),
        file_name: record.file_name.clone(),
        extension: record.extension.clone(),
        preview_state: record.preview_state,
        meta_path: record.meta_path.to_string_lossy().into_owned(),
        preview_artifact_path: record.preview_artifact_path.to_string_lossy().into_owned(),
        source_mtime_unix_ms: record.source_mtime_unix_ms,
        source_hash: record.source_hash.clone(),
        dirty: record.dirty,
        diagnostics: record.diagnostics.clone(),
        direct_reference_uuids: Vec::new(),
    }
}
