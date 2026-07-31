use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::{Transform, Vec3};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;

use super::support::{cube_id, test_state};

#[test]
fn scene_command_capture_does_not_mutate_and_uses_transaction_history() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original_name = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().name.clone());

    let command = state
        .world
        .with_world(|scene| EditorCommand::rename_node(scene, cube, "Captured".to_string()))
        .expect("rename capture should be valid")
        .expect("rename capture should produce a command");

    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );

    state
        .execute_scene_command("Rename scene node", command)
        .expect("shared transaction should apply captured command");
    let transaction_history = state
        .transactions()
        .history_status(HistoryContextId::Global)
        .expect("transaction history should be readable");
    assert_eq!(transaction_history.len, 1);
    assert!(transaction_history.can_undo);
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
    let cube = cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let start = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().transform);

    state.begin_gizmo_transaction().unwrap();
    state.world.with_world_mut(|scene| {
        let _ = scene.update_transform(cube, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)));
    });
    state.finish_gizmo_transaction().unwrap();

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform)
            .translation,
        Vec3::new(2.0, 0.0, 0.0)
    );
}

#[test]
fn gizmo_drag_one_hundred_frames_merges_into_one_transaction_history_record() {
    let mut state = test_state();
    let cube = cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.begin_gizmo_transaction().unwrap();

    for frame in 1..=100 {
        state.world.with_world_mut(|scene| {
            let _ = scene.update_transform(
                cube,
                Transform::from_translation(Vec3::new(frame as f32, 0.0, 0.0)),
            );
        });
        assert!(state.record_gizmo_transaction_step().unwrap());
        let _ = state.snapshot();
    }

    assert!(state.finish_gizmo_transaction().unwrap());
    let transaction_history = state
        .transactions()
        .history_status(HistoryContextId::Global)
        .unwrap();
    assert_eq!(transaction_history.len, 1);
    let details = state
        .transactions()
        .history_details(HistoryContextId::Global, None, 1)
        .unwrap();
    assert_eq!(details.records()[0].command_count, 1);
}

#[test]
fn editor_snapshot_consumes_the_stable_runtime_inspection_artifact() {
    let state = test_state();
    let cube = cube_id(&state);
    let expected_cube_name = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().name.clone());
    let before = state
        .world
        .with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(before.hierarchy_builds(), 0);

    let first = state.snapshot();
    let after_first = state
        .world
        .with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(after_first.hierarchy_builds(), 1);
    assert_eq!(
        after_first.hierarchy_rows_built() as usize,
        first.scene_entries.len()
    );
    assert_eq!(
        first
            .scene_entries
            .iter()
            .find(|entry| entry.id == cube)
            .map(|entry| entry.name.as_str()),
        Some(expected_cube_name.as_str())
    );

    let second = state.snapshot();
    let after_second = state
        .world
        .with_world(|scene| scene.inspection_artifact_diagnostics());
    assert_eq!(after_second, after_first);
    assert_eq!(
        second
            .scene_entries
            .iter()
            .map(|entry| (entry.id, entry.depth, entry.selected))
            .collect::<Vec<_>>(),
        first
            .scene_entries
            .iter()
            .map(|entry| (entry.id, entry.depth, entry.selected))
            .collect::<Vec<_>>()
    );
}
