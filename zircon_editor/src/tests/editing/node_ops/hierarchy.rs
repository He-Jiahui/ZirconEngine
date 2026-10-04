use zircon_runtime::scene::components::NodeKind;

use crate::core::editing::engine::EditCommandError;
use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_and_camera, test_state};
use crate::ui::workbench::state::EditorStateOperationError;

#[test]
fn rename_and_reparent_are_undoable() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);

    assert!(state
        .apply_intent(EditorIntent::RenameNode(cube, "Hero Cube".to_string()))
        .unwrap());
    assert!(state
        .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
        .unwrap());

    state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Hero Cube");
        assert_eq!(node.parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());

    state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_ne!(node.name, "Hero Cube");
        assert_eq!(node.parent, None);
    });
}

#[test]
fn reparenting_multiple_nodes_commits_and_undoes_as_one_transaction() {
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

    assert!(state
        .apply_intent(EditorIntent::SetParents(
            vec![cube, second_cube],
            Some(camera),
        ))
        .unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(second_cube).unwrap().parent, Some(camera));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}

#[test]
fn reparenting_selected_parent_and_child_preserves_the_subtree() {
    let mut state = test_state();
    let (parent, new_parent) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    let child = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube) && node.id != parent)
            .map(|node| node.id)
            .unwrap()
    });
    assert!(state
        .apply_intent(EditorIntent::SetParent(child, Some(parent)))
        .unwrap());

    assert!(state
        .apply_intent(EditorIntent::SetParents(
            vec![parent, child],
            Some(new_parent),
        ))
        .unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, Some(new_parent));
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(parent).unwrap().parent, None);
        assert_eq!(scene.find_node(child).unwrap().parent, Some(parent));
    });
}

#[test]
fn reparenting_multiple_nodes_cancels_the_whole_transaction_on_a_cycle() {
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
    assert!(state
        .apply_intent(EditorIntent::SetParent(cube, Some(camera)))
        .unwrap());

    let error = state
        .apply_intent(EditorIntent::SetParents(
            vec![second_cube, camera],
            Some(cube),
        ))
        .unwrap_err();

    assert!(matches!(
        &error,
        EditorStateOperationError::EditCommand(EditCommandError::ExternalEffect { source })
            if source.to_string().contains("cycle")
    ));
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().parent, Some(camera));
        assert_eq!(scene.find_node(camera).unwrap().parent, None);
        assert_eq!(scene.find_node(second_cube).unwrap().parent, None);
    });
}
