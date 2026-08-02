use crate::core::editing::intent::EditorIntent;
use crate::scene::selection::WorldDomain;
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::EditorState;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::super::support::{cube_and_camera, cube_id, test_state};
use super::viewport::begin_moved_gizmo_drag;

#[test]
fn play_mode_restores_edit_world_and_history_on_exit() {
    let manager = DefaultLevelManager::default();
    let mut state = EditorState::project(
        manager.create_default_level(),
        UVec2::new(1280, 720),
        "sandbox-project",
    );
    let cube = cube_id(&state);
    let original_name = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().name.clone());

    assert!(state.apply_intent(EditorIntent::SelectNode(cube)).unwrap());
    assert!(
        state
            .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_string()))
            .unwrap()
    );
    let edit_world = state.world.with_world(|scene| scene.clone());
    assert!(state.snapshot().can_undo);

    assert!(state.enter_play_mode().unwrap());
    assert!(state.is_playing());
    assert!(!state.scene_viewport_settings().gizmos_enabled);
    let play_snapshot = state.snapshot();
    assert_eq!(play_snapshot.session_mode, EditorSessionMode::Playing);
    assert!(!play_snapshot.can_undo);

    let play_edit_error = state
        .apply_intent(EditorIntent::RenameNode(cube, "Runtime Cube".to_string()))
        .expect_err("play-world changes must not enter edit history");
    assert!(play_edit_error.contains("disabled during play mode"));
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Edited Cube"
    );
    assert!(!state.snapshot().can_undo);

    assert!(state.exit_play_mode().unwrap());
    assert!(!state.is_playing());
    assert!(state.scene_viewport_settings().gizmos_enabled);
    let restored_snapshot = state.snapshot();
    assert_eq!(restored_snapshot.session_mode, EditorSessionMode::Project);
    assert!(restored_snapshot.can_undo);
    assert_eq!(state.world.with_world(|scene| scene.clone()), edit_world);
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn import_is_rejected_during_play_without_poisoning_edit_history() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original_name = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().name.clone());
    assert!(
        state
            .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_string()))
            .unwrap()
    );
    let edit_node_count = state.world.snapshot().node_records().len();

    assert!(state.enter_play_mode().unwrap());
    let error = state
        .import_mesh_asset(
            ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/play-mode.obj",
            )),
            ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/play-mode.zmaterial",
            )),
            "res://models/play-mode.obj",
        )
        .expect_err("imports must not bypass play-mode scene isolation");
    assert!(error.contains("disabled during play mode"));
    assert_eq!(state.world.snapshot().node_records().len(), edit_node_count);

    assert!(state.exit_play_mode().unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn play_mode_restores_the_complete_dual_domain_selection_model() {
    let manager = DefaultLevelManager::default();
    let mut state = EditorState::project(
        manager.create_default_level(),
        UVec2::new(1280, 720),
        "sandbox-project",
    );
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [camera, cube],
        Some(cube)
    ));
    assert!(
        state
            .viewport_controller
            .selection_mut()
            .select_only(WorldDomain::Play, camera)
    );
    let selection_before_play = state.viewport_controller.selection().clone();

    assert!(state.enter_play_mode().unwrap());
    assert_eq!(
        state.viewport_controller.selection().active_domain(),
        WorldDomain::Play
    );
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera, cube]
    );
    assert!(
        state
            .viewport_controller
            .selection_mut()
            .select_only(WorldDomain::Edit, camera)
    );
    assert!(
        state
            .viewport_controller
            .selection_mut()
            .select_only(WorldDomain::Play, cube)
    );

    assert!(state.exit_play_mode().unwrap());
    assert_eq!(
        state.viewport_controller.selection(),
        &selection_before_play
    );
}

#[test]
fn entering_play_during_gizmo_drag_restores_the_edit_transform() {
    let mut state = test_state();
    let (cube, initial) = begin_moved_gizmo_drag(&mut state);

    assert!(state.enter_play_mode().unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        initial
    );
    assert!(!state.viewport_controller.is_handle_drag_active());
    assert!(state.exit_play_mode().unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        initial
    );
}

#[test]
fn play_mode_rejects_unloaded_welcome_world() {
    let mut state = EditorState::welcome(UVec2::new(1280, 720), WelcomePaneSnapshot::default());

    let error = state
        .enter_play_mode()
        .expect_err("welcome mode has no world to play");

    assert_eq!(error, "No project open");
    assert!(!state.is_playing());
    let snapshot = state.snapshot();
    assert_eq!(snapshot.session_mode, EditorSessionMode::Welcome);
    assert!(snapshot.status_line.contains("No project open"));
}
