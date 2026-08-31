use zircon_runtime::scene::components::NodeKind;

use crate::core::editing::engine::EditCommandError;
use crate::core::editing::intent::EditorIntent;
use crate::ui::workbench::state::EditorStateOperationError;

use super::support::{cube_and_camera, test_state};

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
