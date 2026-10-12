use std::sync::Arc;

use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;

use super::update_asset_in_catalog_generation;
use crate::ui::host::editor_asset_manager::manager::DefaultEditorAssetManager;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
};

fn asset(uuid: &str, locator: &str, preview_state: PreviewState) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: uuid.to_string(),
        id: locator.to_string(),
        locator: locator.to_string(),
        kind: ResourceKind::Texture,
        display_name: locator.to_string(),
        file_name: locator.to_string(),
        extension: "png".to_string(),
        preview_state,
        meta_path: format!("{locator}.zmeta"),
        preview_artifact_path: format!("{locator}.preview.png"),
        source_mtime_unix_ms: 0,
        source_hash: "digest".to_string(),
        dirty: preview_state == PreviewState::Dirty,
        diagnostics: Vec::new(),
        direct_reference_uuids: Vec::new(),
    }
}

#[test]
fn stable_catalog_queries_share_the_same_generation_allocation() {
    let manager = DefaultEditorAssetManager::new();
    let first = manager.catalog_snapshot_record();
    let second = manager.catalog_snapshot_record();

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn preview_update_replaces_only_the_matching_generation_row() {
    let unchanged = asset("unchanged", "res://unchanged.png", PreviewState::Ready);
    let stale = asset("updated", "res://updated.png", PreviewState::Dirty);
    let current = Arc::new(EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            catalog_revision: 7,
            assets: vec![unchanged.clone(), stale],
            ..EditorAssetCatalogSnapshotRecord::default()
        },
        11,
    ));
    let updated = asset("updated", "res://updated.png", PreviewState::Ready);

    let next = update_asset_in_catalog_generation(&current, updated.clone(), 12);

    assert_eq!(next.catalog_revision, 7);
    assert_eq!(next.publish_epoch, 12);
    assert_eq!(next.assets[0].as_ref(), &unchanged);
    assert_eq!(next.assets[1].as_ref(), &updated);
    assert!(Arc::ptr_eq(&next.assets[0], &current.assets[0]));
    assert_eq!(current.assets[1].preview_state, PreviewState::Dirty);
}

#[test]
fn ten_thousand_asset_preview_update_preserves_9999_row_allocations_and_indexes() {
    let assets = (0..10_000)
        .map(|index| {
            asset(
                &format!("asset-{index:05}"),
                &format!("res://textures/asset-{index:05}.png"),
                PreviewState::Dirty,
            )
        })
        .collect::<Vec<_>>();
    let current = Arc::new(EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            catalog_revision: 9,
            assets,
            ..EditorAssetCatalogSnapshotRecord::default()
        },
        20,
    ));
    let updated = asset(
        "asset-05000",
        "res://textures/asset-05000.png",
        PreviewState::Ready,
    );

    let next = update_asset_in_catalog_generation(&current, updated, 21);

    assert_eq!(next.assets.len(), 10_000);
    assert_eq!(
        next.asset("asset-05000")
            .expect("updated index")
            .preview_state,
        PreviewState::Ready
    );
    assert_eq!(
        next.asset_by_locator("res://textures/asset-09999.png")
            .expect("locator index")
            .uuid,
        "asset-09999"
    );
    let shared_rows = current
        .assets
        .iter()
        .zip(next.assets.iter())
        .filter(|(left, right)| Arc::ptr_eq(left, right))
        .count();
    assert_eq!(shared_rows, 9_999);
}
