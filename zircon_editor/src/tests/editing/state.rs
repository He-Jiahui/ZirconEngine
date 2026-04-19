use crate::ui::ViewportCommand;
use crate::scene::viewport::{
    HandleElementExtract, OverlayAxis, ProjectionMode, SceneViewportTool, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{UVec2, Vec2};
use zircon_runtime::core::resource::{ResourceKind, ResourceState};
use zircon_runtime::scene::DefaultLevelManager;

use super::asset_workspace::{sample_catalog, sample_material_details, sample_resource_status};
use super::support::{cube_and_camera, cube_id, test_state};
use crate::EditorSessionMode;
use crate::EditorState;

#[test]
fn editor_state_snapshot_projects_structured_asset_workspace() {
    let mut state = test_state();
    state.sync_asset_catalog(sample_catalog());
    state.sync_asset_resources(vec![
        sample_resource_status(
            "res://materials/grid.material.toml",
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
    assert!(state.viewport_controller.selected_node().is_none());
}

#[test]
fn editor_state_with_default_selection_preserves_editor_authored_selection() {
    let manager = DefaultLevelManager::default();
    let state =
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720));

    let snapshot = state.snapshot();

    assert!(snapshot.inspector.is_some());
    assert!(state.viewport_controller.selected_node().is_some());
}

#[test]
fn drag_tool_click_selects_renderable_without_handle_overlay() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state
        .apply_intent(crate::EditorIntent::SelectNode(camera))
        .unwrap();

    let _ = state.apply_viewport_command(&ViewportCommand::SetTool(SceneViewportTool::Drag));

    let cursor = project_entity_cursor(&state, cube, zircon_runtime::core::math::Vec3::new(0.55, 0.0, 0.0));
    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: cursor.x,
        y: cursor.y,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert_eq!(state.viewport_controller.selected_node(), Some(cube));
    assert!(state.render_snapshot().unwrap().overlays.handles.is_empty());
}

#[test]
fn viewport_clicking_light_gizmo_selects_light_node() {
    let mut state = test_state();
    let light = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    node.kind,
                    zircon_runtime::scene::components::NodeKind::DirectionalLight
                )
            })
            .map(|node| node.id)
            .expect("directional light")
    });

    let cursor = {
        let packet = state.render_snapshot().expect("render packet");
        let icon = packet
            .overlays
            .scene_gizmos
            .iter()
            .find(|gizmo| gizmo.owner == light)
            .and_then(|gizmo| gizmo.icons.first())
            .expect("light gizmo icon");
        project_world_position(
            &packet.scene.camera,
            state.viewport_state().size,
            icon.position,
        )
        .expect("light gizmo cursor")
    };

    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: cursor.x,
        y: cursor.y,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert_eq!(state.viewport_controller.selected_node(), Some(light));
}

#[test]
fn render_frame_extract_matches_legacy_render_snapshot_projection() {
    let state = test_state();

    let snapshot = state.render_snapshot().expect("render snapshot");
    let extract = state.render_frame_extract().expect("render frame extract");

    assert_eq!(extract.to_scene_snapshot(), snapshot);
}

#[test]
fn viewport_handle_drag_collapses_into_single_undoable_command() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let start = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().transform);

    let _ = state.apply_viewport_command(&ViewportCommand::SetTool(SceneViewportTool::Move));

    let (press, release) = move_handle_drag_cursor_pair(&state, cube);
    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: press.x,
        y: press.y,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::PointerMoved {
        x: release.x,
        y: release.y,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    let after_drag = state.snapshot();
    assert!(after_drag.can_undo);
    assert!(!after_drag.can_redo);
    assert_ne!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );

    assert!(state.apply_intent(crate::EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
    let after_undo = state.snapshot();
    assert!(!after_undo.can_undo);
    assert!(after_undo.can_redo);

    assert!(state.apply_intent(crate::EditorIntent::Redo).unwrap());
    assert_ne!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
}

fn project_entity_cursor(state: &EditorState, entity: u64, offset: zircon_runtime::core::math::Vec3) -> Vec2 {
    let packet = state.render_snapshot().expect("render packet");
    let transform = state
        .world
        .with_world(|scene| scene.world_transform(entity).unwrap());
    project_world_position(
        &packet.scene.camera,
        state.viewport_state().size,
        transform.matrix().transform_point3(offset),
    )
    .expect("entity cursor")
}

fn move_handle_drag_cursor_pair(state: &EditorState, cube: u64) -> (Vec2, Vec2) {
    let packet = state.render_snapshot().expect("render packet");
    let handle = packet
        .overlays
        .handles
        .iter()
        .find(|handle| handle.owner == cube)
        .expect("move handle");
    let (start, end) = handle
        .elements
        .iter()
        .find_map(|element| match element {
            HandleElementExtract::AxisLine {
                axis, start, end, ..
            } if *axis == OverlayAxis::X => Some((*start, *end)),
            _ => None,
        })
        .expect("x axis handle");
    let start_cursor =
        project_world_position(&packet.scene.camera, state.viewport_state().size, start)
            .expect("axis start");
    let end_cursor = project_world_position(&packet.scene.camera, state.viewport_state().size, end)
        .expect("axis end");
    let direction = (end_cursor - start_cursor).normalize_or_zero();
    let press = start_cursor + direction * 24.0;
    let release = press + direction * 96.0;
    (press, release)
}

fn project_world_position(
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
    world: zircon_runtime::core::math::Vec3,
) -> Option<Vec2> {
    let aspect = viewport.x as f32 / viewport.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => {
            zircon_runtime::core::math::perspective(camera.fov_y_radians, aspect, camera.z_near, camera.z_far)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            zircon_runtime::core::math::Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let clip = projection * zircon_runtime::core::math::view_matrix(camera.transform) * world.extend(1.0);
    if clip.w <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some(Vec2::new(
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
    ))
}
