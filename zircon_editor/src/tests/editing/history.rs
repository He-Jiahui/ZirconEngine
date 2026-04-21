use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;

use crate::core::editing::intent::EditorIntent;

use super::support::{cube_id, test_state};

#[test]
fn undo_redo_restores_created_nodes() {
    let mut state = test_state();
    let initial_count = state.world.snapshot().nodes().len();

    assert!(state
        .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
        .unwrap());
    assert_eq!(state.world.snapshot().nodes().len(), initial_count + 1);

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(state.world.snapshot().nodes().len(), initial_count);

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(state.world.snapshot().nodes().len(), initial_count + 1);
}

#[test]
fn gizmo_drag_is_undone_via_transform_command() {
    let mut state = test_state();
    let cube = cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let start = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().transform);

    state.apply_intent(EditorIntent::BeginGizmoDrag).unwrap();
    state.world.with_world_mut(|scene| {
        let _ = scene.update_transform(cube, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)));
    });
    state.apply_intent(EditorIntent::EndGizmoDrag).unwrap();

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
