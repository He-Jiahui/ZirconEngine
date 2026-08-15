use crate::core::editing::intent::EditorIntent;
use crate::scene::selection::WorldDomain;
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::state::EditorState;
use zircon_runtime::scene::{DefaultLevelManager, NodeKind};
use zircon_runtime_interface::math::UVec2;

use super::super::support::{cube_and_camera, test_state};

#[test]
fn editor_state_new_starts_in_welcome_mode_without_default_selection() {
    let manager = DefaultLevelManager::default();
    let state = EditorState::new(manager.create_default_level(), UVec2::new(1280, 720));

    let snapshot = state.snapshot();

    assert!(!snapshot.project_open);
    assert_eq!(snapshot.session_mode, EditorSessionMode::Welcome);
    assert!(snapshot.inspector.is_none());
    assert!(state
        .viewport_controller
        .selection()
        .active_primary()
        .is_none());
}

#[test]
fn editor_state_with_default_selection_preserves_editor_authored_selection() {
    let manager = DefaultLevelManager::default();
    let state =
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720));

    let snapshot = state.snapshot();

    assert!(snapshot.inspector.is_some());
    assert!(state
        .viewport_controller
        .selection()
        .active_primary()
        .is_some());
}

#[test]
fn editor_state_project_selects_the_default_cube_for_initial_inspection() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    let cube = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .expect("default project scene should contain a cube")
            .id
    });
    let state = EditorState::project(level, UVec2::new(1280, 720), "sandbox-project");

    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
    assert_eq!(
        state
            .snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.id),
        Some(cube)
    );
}

#[test]
fn non_selection_edit_preserves_active_multi_selection() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [camera, cube],
        Some(cube)
    ));

    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Edited Cube".to_string()))
        .unwrap());

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
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
    let snapshot = state.snapshot();
    let selected_entries = snapshot
        .scene_entries
        .iter()
        .filter(|entry| snapshot.scene_entries.is_selected(entry.entity))
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    assert_eq!(selected_entries.len(), 2);
    assert!(selected_entries.contains(&camera));
    assert!(selected_entries.contains(&cube));
    assert_eq!(
        snapshot.inspector.as_ref().map(|inspector| inspector.id),
        Some(cube)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
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
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
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
}

#[test]
fn deleting_from_multi_selection_preserves_surviving_entities() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [camera, cube],
        Some(cube)
    ));

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());

    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera]
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(camera)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
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
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera]
    );
}

#[test]
fn editor_state_snapshot_ignores_stale_editor_selection() {
    let mut state = test_state();
    state
        .viewport_controller
        .selection_mut()
        .select_only_active(999_999);

    let snapshot = state.snapshot();

    assert!(snapshot.inspector.is_none());
    assert!(snapshot
        .scene_entries
        .iter()
        .all(|entry| !snapshot.scene_entries.is_selected(entry.entity)));
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(999_999)
    );
}
