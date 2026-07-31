mod play_mode;
mod selection;
mod viewport;

use std::sync::Arc;

use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::asset_workspace::{sample_catalog, sample_material_details, sample_resource_status};
use super::support::test_state;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};

#[test]
fn editor_state_snapshot_projects_structured_asset_workspace() {
    let mut state = test_state();
    state.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(sample_catalog(), 1),
    ));
    state.sync_asset_resources(vec![
        sample_resource_status(
            "res://materials/grid.zmaterial",
            ResourceKind::Material,
            4,
            ResourceState::Ready,
        ),
        sample_resource_status(
            "res://scenes/main.scene.toml",
            ResourceKind::Scene,
            7,
            ResourceState::Reloading,
        ),
    ]);
    state.select_asset_folder("res://materials");
    state.select_asset(Some("11111111-1111-1111-1111-111111111111".to_string()));
    state.sync_asset_details(Some(Arc::new(EditorAssetDetailsGeneration::from(
        sample_material_details(),
    ))));

    let snapshot = state.snapshot();

    assert_eq!(snapshot.project_overview.project_name, "Sandbox");
    assert_eq!(
        snapshot.asset_activity.selected_folder_id.as_deref(),
        Some("res://materials")
    );
    assert_eq!(
        snapshot.asset_activity.selected_asset_uuid.as_deref(),
        Some("11111111-1111-1111-1111-111111111111")
    );
    assert_eq!(snapshot.asset_activity.visible_assets.len(), 1);
    assert_eq!(
        snapshot.asset_activity.selection.references[0].locator,
        "res://textures/checker.png"
    );
    assert_eq!(snapshot.asset_activity.selection.resource_revision, Some(4));
    assert_eq!(
        snapshot.asset_browser.selected_asset_uuid,
        snapshot.asset_activity.selected_asset_uuid
    );
}

#[test]
fn editor_state_asset_navigation_retargets_both_asset_surfaces() {
    let mut state = test_state();
    state.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(sample_catalog(), 1),
    ));
    state.select_asset_folder("res://materials");
    state.select_asset(Some("11111111-1111-1111-1111-111111111111".to_string()));
    state.sync_asset_details(Some(Arc::new(EditorAssetDetailsGeneration::from(
        sample_material_details(),
    ))));

    state.navigate_to_asset("22222222-2222-2222-2222-222222222222");

    let snapshot = state.snapshot();

    assert_eq!(
        snapshot.asset_activity.selected_folder_id.as_deref(),
        Some("res://scenes")
    );
    assert_eq!(
        snapshot.asset_activity.selected_asset_uuid.as_deref(),
        Some("22222222-2222-2222-2222-222222222222")
    );
    assert_eq!(
        snapshot.asset_browser.selection.locator,
        "res://scenes/main.scene.toml"
    );
    assert!(snapshot.asset_browser.selection.references.is_empty());
}
