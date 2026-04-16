use zircon_manager::{
    AssetChangeKind, AssetChangeRecord, EditorAssetChangeKind, EditorAssetChangeRecord,
    ResourceChangeKind, ResourceChangeRecord,
};

use crate::host::slint_host::{plan_asset_backend_refresh, AssetBackendRefreshPlan};

#[test]
fn asset_backend_refresh_plan_is_idle_without_backend_events() {
    let plan = plan_asset_backend_refresh(None, None, &[], &[], &[]);

    assert_eq!(plan, AssetBackendRefreshPlan::default());
}

#[test]
fn preview_change_only_syncs_catalog_without_touching_runtime_resources() {
    let plan = plan_asset_backend_refresh(
        Some("11111111-1111-1111-1111-111111111111"),
        None,
        &[],
        &[EditorAssetChangeRecord {
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
            mark_presentation_dirty: true,
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
        &[EditorAssetChangeRecord {
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

#[test]
fn default_scene_resource_change_requests_reload_and_runtime_sync() {
    let plan = plan_asset_backend_refresh(
        None,
        Some("res://scenes/main.scene.toml"),
        &[AssetChangeRecord {
            kind: AssetChangeKind::Modified,
            uri: "res://scenes/main.scene.toml".to_string(),
            previous_uri: None,
        }],
        &[],
        &[ResourceChangeRecord {
            kind: ResourceChangeKind::Updated,
            id: "resource::scene".to_string(),
            locator: Some("res://scenes/main.scene.toml".to_string()),
            previous_locator: None,
            revision: 9,
        }],
    );

    assert_eq!(
        plan,
        AssetBackendRefreshPlan {
            sync_resources: true,
            reload_default_scene: true,
            mark_render_dirty: true,
            mark_presentation_dirty: true,
            ..AssetBackendRefreshPlan::default()
        }
    );
}
