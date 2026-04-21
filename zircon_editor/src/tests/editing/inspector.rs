use zircon_runtime::core::math::Vec3;

use crate::core::editing::intent::EditorIntent;

use super::support::{cube_and_camera, cube_id, test_state};

#[test]
fn inspector_batch_commit_groups_name_parent_and_transform() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.update_name_field("Batch Cube".to_string());
    state.update_parent_field(camera.to_string());
    state.update_translation_field(0, "4.0".to_string());
    state.update_translation_field(1, "5.0".to_string());
    state.update_translation_field(2, "6.0".to_string());

    assert!(state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .unwrap());
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Batch Cube");
        assert_eq!(node.parent, Some(camera));
        assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_ne!(node.name, "Batch Cube");
        assert_eq!(node.parent, None);
        assert_ne!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    });
}

#[test]
fn inspector_batch_commit_is_atomic_on_invalid_parent() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().clone());

    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.update_name_field("Should Not Apply".to_string());
    state.update_parent_field("999999".to_string());
    state.update_translation_field(0, "9.0".to_string());
    state.update_translation_field(1, "8.0".to_string());
    state.update_translation_field(2, "7.0".to_string());

    let error = state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .unwrap_err();

    assert!(error.contains("missing parent node"));
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, original.name);
        assert_eq!(node.parent, original.parent);
        assert_eq!(node.transform, original.transform);
    });
}
