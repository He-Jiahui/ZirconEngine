use zircon_runtime::scene::components::NodeKind;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::ui::binding::ViewportCommand;
use crate::ui::workbench::state::EditorStateOperationError;

use super::{
    begin_moved_gizmo_drag, move_handle_drag_cursor_pair,
    support::{cube_id, test_state},
};

#[test]
fn scene_command_capture_does_not_mutate_and_uses_transaction_history() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original_name = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone());

    let command = state
        .world
        .expect_with_world(|scene| EditorCommand::rename_node(scene, cube, "Captured".to_string()))
        .expect("rename capture should be valid")
        .expect("rename capture should produce a command");

    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );

    state
        .execute_scene_command("Rename scene node", command)
        .expect("shared transaction should apply captured command");
    let transaction_history = state
        .transactions()
        .history_status(state.scene_history_context().unwrap())
        .expect("transaction history should be readable");
    assert_eq!(transaction_history.len, 1);
    assert!(transaction_history.can_undo);
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .expect("global history should remain readable")
            .len,
        0
    );
}

#[test]
fn undo_redo_restores_created_nodes() {
    let mut state = test_state();
    let initial_count = state.world.snapshot().node_records().len();

    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    assert_eq!(
        state.world.snapshot().node_records().len(),
        initial_count + 1
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(state.world.snapshot().node_records().len(), initial_count);

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state.world.snapshot().node_records().len(),
        initial_count + 1
    );
}

#[test]
fn gizmo_drag_is_undone_via_transform_command() {
    let mut state = test_state();
    let (cube, start) = begin_moved_gizmo_drag(&mut state);
    let moved = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().transform);
    state
        .apply_viewport_command(&ViewportCommand::LeftReleased)
        .unwrap();

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().transform),
        moved
    );
}

#[test]
fn gizmo_drag_one_hundred_frames_merges_into_one_transaction_history_record() {
    let mut state = test_state();
    let (cube, _) = begin_moved_gizmo_drag(&mut state);
    let (press, first_preview) = move_handle_drag_cursor_pair(&state, cube);
    let drag_delta = first_preview - press;

    for frame in 2..=100 {
        let cursor = press + drag_delta * frame as f32;
        state
            .apply_viewport_command(&ViewportCommand::PointerMoved {
                x: cursor.x,
                y: cursor.y,
            })
            .unwrap();
        let _ = state.snapshot();
    }

    state
        .apply_viewport_command(&ViewportCommand::LeftReleased)
        .unwrap();
    let transaction_history = state
        .transactions()
        .history_status(state.scene_history_context().unwrap())
        .unwrap();
    assert_eq!(transaction_history.len, 1);
    let details = state
        .transactions()
        .history_details(state.scene_history_context().unwrap(), None, 1)
        .unwrap();
    assert_eq!(details.records()[0].command_count, 1);
}

#[test]
fn scene_edits_require_a_lifecycle_committed_document_binding() {
    let mut state = crate::ui::workbench::state::EditorState::new(
        zircon_runtime::scene::DefaultLevelManager::default().create_default_level(),
        zircon_runtime_interface::math::UVec2::new(1280, 720),
    );
    state.mark_project_open();
    let selection_before = state
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
        .unwrap()
        .unwrap();

    assert_eq!(
        state.apply_intent(EditorIntent::CreateNode(NodeKind::Cube)),
        Err(EditorStateOperationError::SceneDocumentNotActive)
    );
    let selection_after = state
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
        .unwrap()
        .unwrap();
    assert_eq!(selection_after, selection_before);
}

#[test]
fn editor_snapshot_consumes_the_stable_runtime_inspection_artifact() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let expected_cube_name = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().name.clone());
    let before = state
        .world
        .expect_with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(before.hierarchy_builds(), 0);

    let first = state.snapshot();
    let after_first = state
        .world
        .expect_with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(after_first.hierarchy_builds(), 1);
    assert_eq!(
        after_first.hierarchy_rows_built() as usize,
        first.scene_entries.len()
    );
    assert_eq!(
        first
            .scene_entries
            .iter()
            .find(|entry| entry.entity == cube)
            .map(|entry| entry.display_name.as_str()),
        Some(expected_cube_name.as_str())
    );
    let runtime_rows = state
        .world
        .expect_with_world(|scene| scene.inspection_artifact().hierarchy_rows_arc());
    assert!(std::sync::Arc::ptr_eq(
        &first.scene_entries.hierarchy_rows_arc(),
        &runtime_rows,
    ));

    let second = state.snapshot();
    let after_second = state
        .world
        .expect_with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(after_second, after_first);
    assert!(first
        .scene_entries
        .shares_hierarchy_with(&second.scene_entries));

    let camera = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Camera))
            .map(|node| node.id)
            .expect("test scene should include a camera")
    });
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .expect("selection should update through the editor intent path");
    let selection_changed = state.snapshot();

    assert!(first
        .scene_entries
        .shares_hierarchy_with(&selection_changed.scene_entries));
    assert!(first.scene_entries.is_selected(cube));
    assert!(selection_changed.scene_entries.is_selected(camera));
    assert!(!selection_changed.scene_entries.is_selected(cube));

    state.world.expect_with_world_mut(|scene| {
        scene
            .rename_node(cube, "Renamed for projection generation")
            .expect("cube rename should advance the inspection generation");
    });
    let changed = state.snapshot();

    assert!(!first
        .scene_entries
        .shares_hierarchy_with(&changed.scene_entries));
    assert_eq!(
        changed
            .scene_entries
            .iter()
            .find(|entry| entry.entity == cube)
            .map(|entry| entry.display_name.as_str()),
        Some("Renamed for projection generation")
    );
}

#[test]
fn editor_snapshot_reuses_the_hierarchy_view_at_thousand_nodes() {
    const NODE_COUNT: usize = 1_000;

    let mut state = test_state();
    let existing_count = state.world.snapshot().node_records().len();
    state.world.expect_with_world_mut(|scene| {
        for _ in existing_count..NODE_COUNT {
            scene
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
        }
    });

    let first = state.snapshot();
    let initial_diagnostics = state
        .world
        .expect_with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(first.scene_entries.len(), NODE_COUNT);
    assert_eq!(initial_diagnostics.hierarchy_builds(), 1);
    assert_eq!(
        initial_diagnostics.hierarchy_rows_built(),
        NODE_COUNT as u64
    );

    for _ in 0..8 {
        let stable = state.snapshot();
        assert!(first
            .scene_entries
            .shares_hierarchy_with(&stable.scene_entries));
    }
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.inspection_artifact_diagnostics()),
        initial_diagnostics
    );

    let camera = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Camera))
            .map(|node| node.id)
            .expect("test scene should include a camera")
    });
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .expect("selection should update through the editor intent path");
    let selection_changed = state.snapshot();

    assert!(first
        .scene_entries
        .shares_hierarchy_with(&selection_changed.scene_entries));
    assert!(selection_changed.scene_entries.is_selected(camera));
    assert_eq!(
        state
            .world
            .expect_with_world(|scene| scene.inspection_artifact_diagnostics()),
        initial_diagnostics
    );
}
