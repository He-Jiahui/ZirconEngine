use zircon_runtime_interface::math::{Transform, Vec3};

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
    state.update_scale_field(0, "2.0".to_string());
    state.update_scale_field(1, "3.0".to_string());
    state.update_scale_field(2, "4.0".to_string());

    assert!(state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .unwrap());
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Batch Cube");
        assert_eq!(node.parent, Some(camera));
        assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(node.transform.scale, Vec3::new(2.0, 3.0, 4.0));
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_ne!(node.name, "Batch Cube");
        assert_eq!(node.parent, None);
        assert_ne!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(node.transform.scale, Vec3::ONE);
    });
}

#[test]
fn inspector_batch_commit_is_atomic_on_invalid_parent() {
    let mut state = test_state();
    let cube = cube_id(&state);
    let original = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap());

    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.update_name_field("Should Not Apply".to_string());
    state.update_parent_field("999999".to_string());
    state.update_translation_field(0, "9.0".to_string());
    state.update_translation_field(1, "8.0".to_string());
    state.update_translation_field(2, "7.0".to_string());

    let error = state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .unwrap_err();

    assert!(
        error.contains("missing parent 999999"),
        "unexpected invalid-parent error: {error}"
    );
    state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, original.name);
        assert_eq!(node.parent, original.parent);
        assert_eq!(node.transform, original.transform);
    });
}

#[test]
fn selected_inspector_snapshot_projects_runtime_artifact_fields() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.world.with_world_mut(|scene| {
        scene.rename_node(cube, "Artifact Cube").unwrap();
        scene.set_parent_checked(cube, Some(camera)).unwrap();
        scene
            .update_transform(
                cube,
                Transform::from_translation(Vec3::new(12.5, -3.0, 7.25))
                    .with_scale(Vec3::new(1.25, 2.5, 0.75)),
            )
            .unwrap();
    });

    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let inspector = state
        .snapshot()
        .inspector
        .expect("selected entity should project an inspector");

    assert_eq!(inspector.id, cube);
    assert_eq!(inspector.name, "Artifact Cube");
    assert_eq!(inspector.parent, camera.to_string());
    assert_eq!(inspector.translation, ["12.50", "-3.00", "7.25"]);
    assert_eq!(inspector.scale, ["1.25", "2.50", "0.75"]);
}
