use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::scene::SceneError;

use crate::core::editing::engine::EditCommandError;
use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_and_camera, cube_id, test_state};
use crate::ui::workbench::state::EditorStateOperationError;

#[test]
fn delete_node_is_undoable() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_none()));

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn delete_undo_keeps_the_inverse_delta_after_restore_preflight_rejection() {
    let mut state = test_state();
    let cube = cube_id(&state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    let replacement = DefaultLevelManager::default().create_default_level();
    let conflict_batch = replacement.with_world_mut(|fixture| {
        assert!(fixture.contains_entity(cube));
        fixture
            .remove_entity_recursive(cube)
            .expect("detach the same-ID fixture node")
    });
    state.world.expect_with_world_mut(|scene| {
        scene
            .restore_detached_entity_batch(conflict_batch)
            .expect("occupy the detached entity id before undo");
    });

    let error = state
        .apply_intent(EditorIntent::Undo)
        .expect_err("the restore preflight must reject the conflicting entity");
    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::SceneMutation {
            operation: "restore detached entity batch",
            source: SceneError::DuplicateEntity { entity },
        }) if entity == cube
    ));

    state.world.expect_with_world_mut(|scene| {
        scene
            .remove_entity(cube)
            .expect("remove the conflicting replacement");
    });
    assert!(
        state.apply_intent(EditorIntent::Undo).unwrap(),
        "the rejected restore must retain the exact move-only inverse delta for retry"
    );
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(cube).is_some()));
}

#[test]
fn deleting_last_camera_is_rejected() {
    let mut state = test_state();
    let camera = state.world.expect_with_world(|scene| scene.active_camera());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(camera))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorStateOperationError::EditCommand(EditCommandError::InvariantViolation {
            invariant: "cannot delete the last remaining camera"
        })
    ));
    assert!(state
        .world
        .expect_with_world(|scene| scene.find_node(camera).is_some()));
}

#[test]
fn deleting_parent_of_the_last_camera_is_rejected() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    assert!(state
        .apply_intent(EditorIntent::SetParent(camera, Some(cube)))
        .unwrap());

    let error = state
        .apply_intent(EditorIntent::DeleteNode(cube))
        .expect_err("deleting a parent must not remove the last camera descendant");

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
