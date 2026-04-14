use crate::EditorIntent;

use super::support::{cube_and_camera, cube_id, test_state};

#[test]
fn delete_node_is_undoable() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(state
        .world
        .with_world(|scene| scene.find_node(cube).is_none()));

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state
        .world
        .with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn deleting_last_camera_is_rejected() {
    let mut state = test_state();
    let camera = state.world.with_world(|scene| scene.active_camera());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(camera))
        .unwrap_err();

    assert!(error.contains("last remaining camera"));
    assert!(state
        .world
        .with_world(|scene| scene.find_node(camera).is_some()));
}

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
