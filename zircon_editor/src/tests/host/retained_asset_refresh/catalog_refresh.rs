use crate::ui::host::editor_asset_manager::{EditorAssetChange, EditorAssetChangeKind};
use crate::ui::retained_host::{plan_asset_backend_refresh, AssetBackendRefreshPlan};

#[test]
fn preview_change_only_syncs_catalog_without_touching_runtime_resources() {
    let plan = plan_asset_backend_refresh(
        Some("11111111-1111-1111-1111-111111111111"),
        None,
        &[],
        &[EditorAssetChange {
            kind: EditorAssetChangeKind::PreviewChanged,
            catalog_revision: 4,
            uuid: Some("11111111-1111-1111-1111-111111111111".to_string()),
            locator: Some("res://materials/grid.material.toml".to_string()),
        }],
        &[],
    );

    assert_eq!(
        plan,
        AssetBackendRefreshPlan {
            sync_catalog: true,
            mark_paint_only_dirty: true,
            ..AssetBackendRefreshPlan::default()
        }
    );
}

#[test]
fn preview_and_catalog_change_keeps_structural_refresh_domains() {
    let plan = plan_asset_backend_refresh(
        Some("11111111-1111-1111-1111-111111111111"),
        None,
        &[],
        &[
            EditorAssetChange {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: 4,
                uuid: Some("11111111-1111-1111-1111-111111111111".to_string()),
                locator: Some("res://materials/grid.material.toml".to_string()),
            },
            EditorAssetChange {
                kind: EditorAssetChangeKind::CatalogChanged,
                catalog_revision: 5,
                uuid: None,
                locator: None,
            },
        ],
        &[],
    );

    assert_eq!(
        plan,
        AssetBackendRefreshPlan {
            sync_catalog: true,
            refresh_selected_asset_details: true,
            refresh_visible_asset_previews: true,
            mark_presentation_dirty: true,
            mark_paint_only_dirty: true,
            ..AssetBackendRefreshPlan::default()
        }
    );
}

#[test]
fn catalog_change_refreshes_details_and_visible_previews_without_resource_sync() {
    let plan = plan_asset_backend_refresh(
        Some("11111111-1111-1111-1111-111111111111"),
        None,
        &[],
        &[EditorAssetChange {
            kind: EditorAssetChangeKind::CatalogChanged,
            catalog_revision: 5,
            uuid: None,
            locator: None,
        }],
        &[],
    );

    assert_eq!(
        plan,
        AssetBackendRefreshPlan {
            sync_catalog: true,
            refresh_selected_asset_details: true,
            refresh_visible_asset_previews: true,
            mark_presentation_dirty: true,
            ..AssetBackendRefreshPlan::default()
        }
    );
}
