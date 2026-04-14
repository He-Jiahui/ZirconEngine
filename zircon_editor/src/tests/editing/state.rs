use zircon_manager::{AssetRecordKind, ResourceStateRecord};
use zircon_math::UVec2;
use zircon_scene::DefaultLevelManager;

use super::asset_workspace::{sample_catalog, sample_material_details, sample_resource_status};
use super::support::test_state;
use crate::EditorSessionMode;
use crate::EditorState;

#[test]
fn editor_state_snapshot_projects_structured_asset_workspace() {
    let mut state = test_state();
    state.sync_asset_catalog(sample_catalog());
    state.sync_asset_resources(vec![
        sample_resource_status(
            "res://materials/grid.material.toml",
            AssetRecordKind::Material,
            4,
            ResourceStateRecord::Ready,
        ),
        sample_resource_status(
            "res://scenes/main.scene.toml",
            AssetRecordKind::Scene,
            7,
            ResourceStateRecord::Reloading,
        ),
    ]);
    state.select_asset_folder("res://materials");
    state.select_asset(Some("11111111-1111-1111-1111-111111111111".to_string()));
    state.sync_asset_details(Some(sample_material_details()));

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
    state.sync_asset_catalog(sample_catalog());
    state.select_asset_folder("res://materials");
    state.select_asset(Some("11111111-1111-1111-1111-111111111111".to_string()));
    state.sync_asset_details(Some(sample_material_details()));

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

#[test]
fn editor_state_new_starts_in_welcome_mode_without_default_selection() {
    let manager = DefaultLevelManager::default();
    let state = EditorState::new(manager.create_default_level(), UVec2::new(1280, 720));

    let snapshot = state.snapshot();

    assert!(!snapshot.project_open);
    assert_eq!(snapshot.session_mode, EditorSessionMode::Welcome);
    assert!(snapshot.inspector.is_none());
    assert!(state.world.with_world(|scene| scene.selected_node().is_none()));
}

#[test]
fn editor_state_with_default_selection_preserves_editor_authored_selection() {
    let manager = DefaultLevelManager::default();
    let state =
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720));

    let snapshot = state.snapshot();

    assert!(snapshot.inspector.is_some());
    assert!(state.world.with_world(|scene| scene.selected_node().is_some()));
}
