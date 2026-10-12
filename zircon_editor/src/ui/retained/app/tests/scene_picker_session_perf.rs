use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::asset::project::PreviewState;

use super::*;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
};

#[test]
fn optimization_wave_20260824q_editor51_scene_picker_sort_preserves_order_and_exact_dedup() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            assets: [
                "res://levels/alpha.scene.toml",
                "res://levels/Alpha.scene.toml",
                "res://levels/zebra.scene.toml",
                "res://levels/alpha.scene.toml",
            ]
            .into_iter()
            .map(catalog_record)
            .collect(),
            ..EditorAssetCatalogSnapshotRecord::default()
        },
        1,
    );

    let entries = scene_entries_from_catalog(&catalog);
    let uris = entries
        .iter()
        .map(ScenePickerEntry::scene_uri)
        .collect::<Vec<_>>();

    assert_eq!(
        uris,
        vec![
            "res://levels/Alpha.scene.toml",
            "res://levels/alpha.scene.toml",
            "res://levels/zebra.scene.toml",
        ]
    );
}

#[test]
fn optimization_wave_20260824q_editor51_scene_picker_sort_borrows_original_uris() {
    let source = include_str!("../scene_picker_session.rs");
    let (production, _) = source
        .rsplit_once("#[cfg(test)]")
        .expect("scene picker optimization test module");
    let compact = production.split_whitespace().collect::<String>();

    assert!(compact.contains("(asset.locator.to_lowercase(),asset.locator.as_str())"));
    assert!(production.contains("scene_uris.sort_unstable()"));
    assert!(production.contains("scene_uris.dedup()"));
    assert!(!production
        .contains("sort_by_cached_key(|scene_uri| (scene_uri.to_lowercase(), scene_uri.clone()))"));
}

#[test]
#[ignore = "managed release evidence"]
fn optimization_wave_20260824q_editor51_scene_picker_sort_projection_evidence() {
    const URI_COUNT: usize = 20_000;
    const SAMPLE_PAIRS: usize = 11;
    const TARGET: Duration = Duration::from_millis(100);

    let locators = (0..URI_COUNT)
        .rev()
        .map(|index| {
            format!(
                "res://world/region-{index:05}/streaming/sublevel/authoring/scene-{index:05}.scene.toml"
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_ns(|| legacy_scene_uri_projection(&locators)));
            optimized_samples.push(measure_ns(|| borrowed_scene_uri_projection(&locators)));
        } else {
            optimized_samples.push(measure_ns(|| borrowed_scene_uri_projection(&locators)));
            legacy_samples.push(measure_ns(|| legacy_scene_uri_projection(&locators)));
        }
    }

    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let locator_bytes = locators.iter().map(String::len).sum::<usize>();
    let string_bytes_before = locator_bytes * 3;
    let string_bytes_after = locator_bytes * 2;
    let string_byte_reduction_percent =
        (1.0 - string_bytes_after as f64 / string_bytes_before as f64) * 100.0;

    assert!(
        optimized_p95 <= TARGET.as_nanos(),
        "optimized_p95_ns={optimized_p95} target_ns={}",
        TARGET.as_nanos()
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90),
        "optimized_p95_ns={optimized_p95} legacy_p95_ns={legacy_p95}"
    );
    println!(
        "EDITOR51_SCENE_PICKER_SORT_BENCH_V1 uri_count={} locator_bytes={} string_bytes_before={} string_bytes_after={} string_byte_reduction_percent={:.4} legacy_p95_ns={} optimized_p95_ns={} target_ns={}",
        URI_COUNT,
        locator_bytes,
        string_bytes_before,
        string_bytes_after,
        string_byte_reduction_percent,
        legacy_p95,
        optimized_p95,
        TARGET.as_nanos()
    );
}

fn legacy_scene_uri_projection(locators: &[String]) -> u64 {
    let mut projected = locators.to_vec();
    projected.sort_by_cached_key(|scene_uri| (scene_uri.to_lowercase(), scene_uri.clone()));
    projected.dedup();
    projection_checksum(&projected)
}

fn borrowed_scene_uri_projection(locators: &[String]) -> u64 {
    let mut projected = locators
        .iter()
        .map(|scene_uri| (scene_uri.to_lowercase(), scene_uri.as_str()))
        .collect::<Vec<_>>();
    projected.sort_unstable();
    projected.dedup();
    let projected = projected
        .into_iter()
        .map(|(_, scene_uri)| scene_uri.to_string())
        .collect::<Vec<_>>();
    projection_checksum(&projected)
}

fn projection_checksum(projected: &[String]) -> u64 {
    black_box(projected.iter().fold(0_u64, |checksum, uri| {
        checksum.wrapping_add(uri.len() as u64)
    }))
}

fn measure_ns(measure: impl FnOnce() -> u64) -> u128 {
    let started = Instant::now();
    black_box(measure());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn catalog_record(locator: &str) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: format!("asset-{locator}"),
        id: locator.to_string(),
        locator: locator.to_string(),
        kind: ResourceKind::Scene,
        display_name: locator.to_string(),
        file_name: locator.to_string(),
        extension: "toml".to_string(),
        preview_state: PreviewState::Dirty,
        meta_path: String::new(),
        preview_artifact_path: String::new(),
        source_mtime_unix_ms: 0,
        source_hash: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        direct_reference_uuids: Vec::new(),
    }
}
