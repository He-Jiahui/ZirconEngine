mod camera_authority;
mod play_mode;
mod selection;
mod viewport;

pub(crate) use viewport::{begin_moved_gizmo_drag, move_handle_drag_cursor_pair};

use std::sync::Arc;

use zircon_runtime::core::resource::ResourceManager;
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::asset_workspace::{sample_catalog, sample_material_details, sample_resource_status};
use super::support::test_state;
use crate::core::editor_event::ConsoleMessageFilter;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};
use crate::ui::workbench::snapshot::EditorConsoleMessageLevel;

#[test]
fn editor_state_snapshot_keeps_latest_status_and_bounded_console_history() {
    let mut state = test_state();

    state.set_status_line("Compiled materials");
    state.set_status_line("Scene ready");

    let snapshot = state.snapshot();
    let history = snapshot.console_output.lines().collect::<Vec<_>>();
    assert_eq!(snapshot.status_line, "Scene ready");
    assert_eq!(
        history[history.len().saturating_sub(2)..],
        ["Compiled materials", "Scene ready"]
    );

    assert!(state.clear_console_history());
    let cleared = state.snapshot();
    assert_eq!(cleared.status_line, "Scene ready");
    assert!(cleared.console_output.is_empty());
    assert!(!state.clear_console_history());

    state.set_status_line("Scene ready");
    assert_eq!(state.snapshot().console_output.as_ref(), "Scene ready");
}

#[test]
fn editor_state_snapshot_preserves_console_message_levels() {
    let mut state = test_state();

    state.clear_console_history();
    state.set_status_line_with_level("Shader fallback", EditorConsoleMessageLevel::Warning);
    state.set_status_line_with_level("Pipeline failed", EditorConsoleMessageLevel::Error);

    let output = state.snapshot().console_output;
    assert_eq!(output.as_ref(), "Shader fallback\nPipeline failed");
    assert_eq!(
        output.levels(),
        &[
            EditorConsoleMessageLevel::Warning,
            EditorConsoleMessageLevel::Error,
        ]
    );
    assert_eq!(output.counts().info, 0);
    assert_eq!(output.counts().warning, 1);
    assert_eq!(output.counts().error, 1);
    assert_eq!(output.counts().total(), 2);
}

#[test]
fn editor_state_console_filter_projects_visible_output_and_total_counts() {
    let mut state = test_state();
    state.clear_console_history();
    state.set_status_line_with_level("Ready", EditorConsoleMessageLevel::Info);
    state.set_status_line_with_level("Shader fallback", EditorConsoleMessageLevel::Warning);
    state.set_status_line_with_level("Pipeline failed", EditorConsoleMessageLevel::Error);

    assert!(state.set_console_message_filter(ConsoleMessageFilter::Error));
    let output = state.snapshot().console_output;
    assert_eq!(output.as_ref(), "Pipeline failed");
    assert_eq!(output.filter(), ConsoleMessageFilter::Error);
    assert_eq!(output.counts().total(), 3);

    assert!(state.set_console_message_filter(ConsoleMessageFilter::All));
    assert_eq!(
        state.snapshot().console_output.as_ref(),
        "Ready\nShader fallback\nPipeline failed"
    );
}

#[test]
fn editor_state_snapshot_projects_structured_asset_workspace() {
    let mut state = test_state();
    state.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(sample_catalog(), 1),
    ));
    let resource_manager = ResourceManager::new();
    for record in [
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
    ] {
        resource_manager.register_record(record);
    }
    state.sync_asset_resources(resource_manager.management_generation());
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
