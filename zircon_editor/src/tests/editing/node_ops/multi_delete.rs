use zircon_runtime::scene::components::NodeKind;

use crate::core::editing::engine::EditCommandError;
use crate::core::editing::intent::EditorIntent;
use crate::scene::selection::WorldDomain;
use crate::tests::editing::support::{cube_and_camera, test_state};
use crate::ui::workbench::state::EditorStateOperationError;

#[test]
fn deleting_multiple_selected_nodes_commits_and_undoes_as_one_transaction() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, second_cube],
        Some(second_cube),
    ));

    assert!(state.delete_selected().unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(second_cube).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(second_cube).is_some());
    });
}

#[test]
fn deleting_multiple_nodes_restores_the_selection_snapshot_on_undo() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let second_cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, second_cube],
        Some(second_cube),
    ));

    assert!(state.delete_selected().unwrap());
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
        [cube, second_cube]
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(second_cube)
    );
}

#[test]
fn deleting_selection_with_the_last_camera_cancels_the_whole_transaction() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, camera],
        Some(cube),
    ));

    let error = state.delete_selected().unwrap_err();

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(camera).is_some());
    });
}

#[test]
fn deleting_multiple_cameras_cancels_the_whole_transaction() {
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
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, first_camera, second_camera],
        Some(cube),
    ));

    let error = state.delete_selected().unwrap_err();

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
fn deleting_selected_parent_and_child_collapses_to_one_subtree_command() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let child = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(child, Some(cube)))
        .unwrap());
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, child],
        Some(cube),
    ));

    assert!(state.delete_selected().unwrap());
    state.world.expect_with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(child).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(child).unwrap().parent, Some(cube));
    });
}
