use serde_json::json;
use zircon_runtime_interface::math::Vec3;
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_and_camera, cube_id, test_state};

use super::support::{
    cloud_layer_descriptor, read_reflected_field, CLOUD_LAYER_TYPE_PATH, HIERARCHY_TYPE_PATH,
    LOCAL_TRANSFORM_TYPE_PATH, NAME_TYPE_PATH,
};

#[test]
fn reflected_editor_command_routes_inspector_fields_through_reflection() {
    let mut state = test_state();
    let (entity, camera) = cube_and_camera(&state);
    let original_name = state
        .world
        .expect_with_world(|scene| read_reflected_field(scene, entity, NAME_TYPE_PATH, "value"));
    let original_parent = state.world.expect_with_world(|scene| {
        read_reflected_field(scene, entity, HIERARCHY_TYPE_PATH, "parent")
    });
    let original_translation = state.world.expect_with_world(|scene| {
        read_reflected_field(scene, entity, LOCAL_TRANSFORM_TYPE_PATH, "translation")
    });
    state.world.expect_with_world_mut(|scene| {
        scene
            .register_component_type(cloud_layer_descriptor())
            .expect("dynamic component descriptor should register");
        scene
            .set_dynamic_component(
                entity,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "thin" }),
            )
            .expect("dynamic component should attach");
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");
    state.update_name_field("  Reflected Cube  ".to_string());
    state.update_parent_field(camera.to_string());
    let _ = state.update_translation_field(0, "3.5".to_string());
    let _ = state.update_translation_field(1, "4.5".to_string());
    let _ = state.update_translation_field(2, "5.5".to_string());
    state.update_dynamic_component_field(
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage"),
        "0.8".to_string(),
    );

    assert!(state
        .apply_inspector_changes()
        .expect("inspector change should apply through reflected command"));
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        ReflectedValue::String("Reflected Cube".to_string())
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        ReflectedValue::Vec3([3.5, 4.5, 5.5])
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.8)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        original_name
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        original_parent
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        original_translation
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.25)
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        ReflectedValue::String("Reflected Cube".to_string())
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        ReflectedValue::Vec3([3.5, 4.5, 5.5])
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.8)
    );
}

#[test]
fn reflected_editor_command_routes_vector_and_entity_text_fields_through_reflection() {
    let mut state = test_state();
    let (entity, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");
    let original_parent = state.world.expect_with_world(|scene| {
        read_reflected_field(scene, entity, HIERARCHY_TYPE_PATH, "parent")
    });
    let original_scale = state.world.expect_with_world(|scene| {
        read_reflected_field(scene, entity, LOCAL_TRANSFORM_TYPE_PATH, "scale")
    });

    state.update_dynamic_component_field(
        format!("{LOCAL_TRANSFORM_TYPE_PATH}.scale"),
        "[2.0, 3.0, 4.0]".to_string(),
    );
    state.update_dynamic_component_field(
        format!("{HIERARCHY_TYPE_PATH}.parent"),
        camera.to_string(),
    );

    assert!(state
        .apply_inspector_changes()
        .expect("vector and entity text edits should apply through reflection"));
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        ReflectedValue::Vec3([2.0, 3.0, 4.0])
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        original_scale
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        original_parent
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        ReflectedValue::Vec3([2.0, 3.0, 4.0])
    );
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );
}

#[test]
fn reflected_editor_command_rejects_invalid_vector_text_without_mutating() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");
    let original_scale = state.world.expect_with_world(|scene| {
        read_reflected_field(scene, entity, LOCAL_TRANSFORM_TYPE_PATH, "scale")
    });

    state.update_dynamic_component_field(
        format!("{LOCAL_TRANSFORM_TYPE_PATH}.scale"),
        "1.0, nope, 3.0".to_string(),
    );

    let error = state
        .apply_inspector_changes()
        .expect_err("invalid vector text should be rejected before mutation");
    assert!(matches!(
        error,
        crate::ui::workbench::state::EditorStateOperationError::Inspector(
            crate::ui::workbench::state::InspectorEditError::InvalidVector {
                type_name: "Vec3",
                component_count: 3,
                ..
            }
        )
    ));
    assert_eq!(
        state.world.expect_with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        original_scale
    );
}

#[test]
fn reflected_editor_command_preserves_reflection_read_error_source() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");
    state.update_dynamic_component_field("missing.Component.value", "1".to_string());

    let error = state
        .apply_inspector_changes()
        .expect_err("unknown reflected component must be rejected before mutation");

    assert!(matches!(
        error,
        crate::ui::workbench::state::EditorStateOperationError::Inspector(
            crate::ui::workbench::state::InspectorEditError::ReflectionRead {
                ref field_id,
                source: ReflectError::UnknownType { ref type_path },
            }
        ) if field_id == "missing.Component.value" && type_path == "missing.Component"
    ));
}

#[test]
fn inspector_no_op_name_and_dynamic_edits_preserve_authoritative_transform_bits() {
    let mut state = test_state();
    let entity = cube_id(&state);
    let translation = Vec3::new(12.3456, -0.0012345, 987.6543);
    let scale = Vec3::new(1.23456, 0.9876543, 4.32109);
    state.world.expect_with_world_mut(|scene| {
        let mut transform = scene.find_node(entity).unwrap().transform;
        transform.translation = translation;
        transform.scale = scale;
        scene.update_transform(entity, transform).unwrap();
        scene
            .register_component_type(cloud_layer_descriptor())
            .unwrap();
        scene
            .set_dynamic_component(
                entity,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "thin" }),
            )
            .unwrap();
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();

    assert!(!state.apply_inspector_changes().unwrap());
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap()
            .len,
        0
    );

    state.update_name_field("Precise Cube".to_owned());
    assert!(state.apply_inspector_changes().unwrap());
    state.update_dynamic_component_field(
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage"),
        "0.8".to_owned(),
    );
    assert!(state.apply_inspector_changes().unwrap());

    state.world.expect_with_world(|scene| {
        let transform = scene.find_node(entity).unwrap().transform;
        for (actual, expected) in transform
            .translation
            .to_array()
            .into_iter()
            .zip(translation.to_array())
            .chain(transform.scale.to_array().into_iter().zip(scale.to_array()))
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        assert_eq!(scene.find_node(entity).unwrap().name, "Precise Cube");
        assert_eq!(
            read_reflected_field(scene, entity, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.8)
        );
    });
}

#[test]
fn actual_selected_scene_rotation_reaches_read_only_authored_inspector_fields() {
    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use zircon_runtime_interface::math::{EulerRot, Quat};
    use zircon_runtime_interface::ui::layout::UiSize;
    let mut state = test_state();
    let entity = cube_id(&state);
    state.world.expect_with_world_mut(|scene| {
        let mut transform = scene.local_transform(entity).unwrap();
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            30_f32.to_radians(),
            10_f32.to_radians(),
            (-20_f32).to_radians(),
        );
        scene.update_transform(entity, transform).unwrap();
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();
    let snapshot = state.snapshot();
    assert_eq!(
        snapshot
            .inspector
            .as_ref()
            .unwrap()
            .rotation_degrees
            .as_ref()
            .unwrap(),
        &["30.00", "10.00", "-20.00"]
    );
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, snapshot.inspector.as_ref())
        .unwrap();
    for (control, expected) in [
        ("WorkbenchTransformRotationX", "30.00 deg"),
        ("WorkbenchTransformRotationY", "10.00 deg"),
        ("WorkbenchTransformRotationZ", "-20.00 deg"),
    ] {
        let node = bridge
            .host_projection()
            .node_by_control_id(control)
            .unwrap();
        assert_eq!(
            node.properties.get("value"),
            Some(&crate::ui::template_runtime::RetainedUiHostValue::String(
                expected.to_string()
            ))
        );
        assert!(node.disabled);
        assert!(
            node.routes.is_empty(),
            "read-only runtime rotation has no authored edit command"
        );
    }
}
