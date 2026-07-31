use zircon_runtime::scene::components::NodeKind;

use crate::scene::selection::WorldDomain;

use crate::core::editing::intent::EditorIntent;

use super::support::{cube_and_camera, cube_id, test_state};

#[test]
fn delete_node_is_undoable() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).is_none())
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).is_some())
    );
}

#[test]
fn deleting_last_camera_is_rejected() {
    let mut state = test_state();
    let camera = state.world.with_world(|scene| scene.active_camera());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(camera))
        .unwrap_err();

    assert!(error.contains("last remaining camera"));
    assert!(
        state
            .world
            .with_world(|scene| scene.find_node(camera).is_some())
    );
}

#[test]
fn rename_and_reparent_are_undoable() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);

    assert!(
        state
            .apply_intent(EditorIntent::RenameNode(cube, "Hero Cube".to_string()))
            .unwrap()
    );
    assert!(
        state
            .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
            .unwrap()
    );

    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Hero Cube");
        assert_eq!(node.parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());

    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_ne!(node.name, "Hero Cube");
        assert_eq!(node.parent, None);
    });
}

#[test]
fn reparenting_multiple_nodes_commits_and_undoes_as_one_transaction() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let second_cube = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });

    assert!(
        state
            .apply_intent(EditorIntent::SetParents(
                vec![cube, second_cube],
                Some(camera),
            ))
            .unwrap()
    );
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(second_cube).unwrap().parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}

#[test]
fn reparenting_selected_parent_and_child_preserves_the_subtree() {
    let mut state = test_state();
    let (parent, new_parent) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let child = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != parent)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(
        state
            .apply_intent(EditorIntent::SetParent(child, Some(parent)))
            .unwrap()
    );

    assert!(
        state
            .apply_intent(EditorIntent::SetParents(
                vec![parent, child],
                Some(new_parent),
            ))
            .unwrap()
    );
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, Some(new_parent));
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, None);
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });
}

#[test]
fn reparenting_multiple_nodes_cancels_the_whole_transaction_on_a_cycle() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let second_cube = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(
        state
            .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
            .unwrap()
    );

    let error = state
        .apply_intent(EditorIntent::SetParents(
            vec![second_cube, camera],
            Some(cube),
        ))
        .unwrap_err();

    assert!(error.contains("cycle"));
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(camera).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}

#[test]
fn deleting_multiple_selected_nodes_commits_and_undoes_as_one_transaction() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let second_cube = state.world.with_world(|scene| {
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
    state.world.with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(second_cube).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(second_cube).is_some());
    });
}

#[test]
fn deleting_multiple_nodes_restores_the_selection_snapshot_on_undo() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let second_cube = state.world.with_world(|scene| {
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

    assert!(error.contains("last remaining camera"));
    state.world.with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(camera).is_some());
    });
}

#[test]
fn deleting_multiple_cameras_cancels_the_whole_transaction() {
    let mut state = test_state();
    let (cube, first_camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Camera))
            .unwrap()
    );
    let second_camera = state.world.with_world(|scene| {
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

    assert!(error.contains("last remaining camera"));
    state.world.with_world(|scene| {
        assert!(scene.find_node(cube).is_some());
        assert!(scene.find_node(first_camera).is_some());
        assert!(scene.find_node(second_camera).is_some());
    });
}

#[test]
fn deleting_selected_parent_and_child_collapses_to_one_subtree_command() {
    let mut state = test_state();
    let (cube, _camera) = cube_and_camera(&state);
    assert!(
        state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap()
    );
    let child = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != cube)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(
        state
            .apply_intent(EditorIntent::SetParent(child, Some(cube)))
            .unwrap()
    );
    assert!(state.viewport_controller.selection_mut().replace(
        WorldDomain::Edit,
        [cube, child],
        Some(cube),
    ));

    assert!(state.delete_selected().unwrap());
    state.world.with_world(|scene| {
        assert!(scene.find_node(cube).is_none());
        assert!(scene.find_node(child).is_none());
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        assert_eq!(scene.find_node(child).unwrap().parent, Some(cube));
    });
}
