use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::ecs::DetachedEntityBatchDiagnostics;
use zircon_runtime::scene::NodeId;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::{EditCommandError, HistoryStatus};
use crate::core::editing::intent::EditorIntent;
use crate::ui::workbench::state::{EditorState, EditorStateOperationError};

use super::support::{cube_and_camera, test_state};

#[derive(Debug, PartialEq)]
struct DeleteRejectionSnapshot {
    world: serde_json::Value,
    generation: u64,
    active_camera: NodeId,
    detached_batches: DetachedEntityBatchDiagnostics,
    selected: Vec<NodeId>,
    primary: Option<NodeId>,
    history: HistoryStatus,
}

impl DeleteRejectionSnapshot {
    fn capture(state: &EditorState) -> Self {
        let (world, generation, active_camera, detached_batches) =
            state.world.expect_with_world(|scene| {
                (
                    serde_json::to_value(scene).unwrap(),
                    scene.world_generation(),
                    scene.active_camera(),
                    scene
                        .ecs_frame_performance_diagnostics()
                        .detached_entity_batches,
                )
            });
        Self {
            world,
            generation,
            active_camera,
            detached_batches,
            selected: state
                .viewport_controller
                .selection()
                .active_items()
                .iter()
                .copied()
                .collect(),
            primary: state.viewport_controller.selection().active_primary(),
            history: state
                .transactions()
                .history_status(state.scene_history_context().unwrap())
                .unwrap(),
        }
    }
}

fn assert_last_camera_rejection(error: EditorStateOperationError) {
    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
}

#[test]
fn deleting_captured_subtree_rechecks_cameras_before_apply() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    let command = state
        .world
        .expect_with_world(|scene| EditorCommand::delete_node(scene, cube).unwrap());
    state.world.expect_with_world_mut(|scene| {
        scene.set_parent_checked(camera, Some(cube)).unwrap();
    });
    let before = DeleteRejectionSnapshot::capture(&state);

    assert_last_camera_rejection(
        state
            .execute_scene_command("Delete captured subtree", command)
            .unwrap_err(),
    );
    assert_eq!(DeleteRejectionSnapshot::capture(&state), before);
}

#[test]
fn deleting_redo_rechecks_cameras_and_retains_history_for_retry() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world_mut(|scene| {
        scene.set_parent_checked(camera, Some(cube)).unwrap();
    });
    let before = DeleteRejectionSnapshot::capture(&state);
    assert!(before.history.can_redo);

    assert_last_camera_rejection(state.apply_intent(EditorIntent::Redo).unwrap_err());
    assert_eq!(DeleteRejectionSnapshot::capture(&state), before);

    state.world.expect_with_world_mut(|scene| {
        scene.set_parent_checked(camera, None).unwrap();
    });
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(!scene.contains_entity(cube));
        assert!(scene.contains_entity(camera));
        assert_eq!(scene.camera_count(), 1);
    });
}

#[test]
#[ignore = "managed Editor03 2/128 camera subtree with 100k unrelated entity gate"]
fn deleting_all_cameras_after_capture_managed_scale_fixture() {
    for camera_count in [2, 128] {
        let mut state = test_state();
        let (cube, last_outside_camera) = cube_and_camera(&state);
        state.world.expect_with_world_mut(|scene| {
            for _ in 1..camera_count {
                let camera = scene.spawn_node(NodeKind::Camera).unwrap();
                scene.set_parent_checked(camera, Some(cube)).unwrap();
            }
            for _ in 0..100_000 {
                scene.spawn_node(NodeKind::Empty).unwrap();
            }
        });
        let command = state.world.expect_with_world(|scene| {
            assert_eq!(scene.camera_count(), camera_count);
            EditorCommand::delete_node(scene, cube).unwrap()
        });
        state.world.expect_with_world_mut(|scene| {
            scene
                .set_parent_checked(last_outside_camera, Some(cube))
                .unwrap();
        });
        let before = DeleteRejectionSnapshot::capture(&state);

        assert_last_camera_rejection(
            state
                .execute_scene_command("Delete all cameras subtree", command)
                .unwrap_err(),
        );
        assert_eq!(DeleteRejectionSnapshot::capture(&state), before);
    }
}

#[test]
fn deleting_from_a_camera_free_scene_is_rejected() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene
            .remove_entity(camera)
            .expect("remove the only camera for the zero-camera preflight case");
        assert_eq!(scene.camera_count(), 0);
    });

    let error = state
        .apply_intent(EditorIntent::DeleteNode(cube))
        .expect_err("a camera-free scene must retain the existing delete rejection semantics");

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn deleting_parent_of_all_cameras_is_rejected() {
    let mut state = test_state();
    let (cube, first_camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Camera))
        .unwrap());
    let second_camera = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Camera) && node.id != first_camera)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(first_camera, Some(cube)))
        .unwrap());
    assert!(state
        .apply_intent(EditorIntent::SetParent(second_camera, Some(cube)))
        .unwrap());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(cube))
        .expect_err("deleting a parent must not remove every camera descendant");

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(first_camera).is_some());
        assert!(scene.find_node(second_camera).is_some());
    });
}

#[test]
fn delete_undo_redo_undo_rebuilds_the_batch_and_restores_active_camera() {
    let mut state = test_state();
    let (cube, first_camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Camera))
        .unwrap());
    let second_camera = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Camera) && node.id != first_camera)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(first_camera, Some(cube)))
        .unwrap());
    state
        .world
        .expect_with_world_mut(|scene| scene.set_active_camera(first_camera));

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(first_camera).is_none());
        assert_eq!(scene.active_camera(), second_camera);
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(first_camera).is_some());
        assert_eq!(scene.active_camera(), first_camera);
    });

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(first_camera).is_none());
        assert_eq!(scene.active_camera(), second_camera);
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(first_camera).is_some());
        assert_eq!(scene.active_camera(), first_camera);
    });
}
