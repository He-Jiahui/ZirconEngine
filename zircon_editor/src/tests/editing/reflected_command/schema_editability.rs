use serde_json::json;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_id, test_state};

use super::support::{
    cloud_layer_descriptor, read_reflected_field, scene_with_cloud_layer, CLOUD_LAYER_TYPE_PATH,
};

#[test]
fn reflected_editor_command_rejects_readonly_dynamic_field() {
    let mut scene = scene_with_cloud_layer();
    let entity = scene
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    scene
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "thin" }),
        )
        .expect("dynamic component should attach");

    let error = EditorCommand::set_reflected_scene_field(
        &scene,
        entity,
        CLOUD_LAYER_TYPE_PATH,
        "label",
        ReflectedValue::String("storm".to_string()),
    )
    .err()
    .expect("read-only reflected dynamic fields must be rejected");

    assert!(error.to_string().contains("not editable"));
    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "label"),
        ReflectedValue::String("thin".to_string())
    );
}

#[test]
fn reflected_editor_command_uses_reflection_schema_for_dynamic_field_editability() {
    let mut state = test_state();
    let entity = cube_id(&state);
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

    assert!(state
        .can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.coverage"))
        .expect("authoring world gateway should succeed"));
    assert!(!state
        .can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.label"))
        .expect("authoring world gateway should succeed"));
    assert!(!state
        .can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.missing"))
        .expect("authoring world gateway should succeed"));
}

#[test]
fn reflected_editor_command_rejects_dynamic_field_when_reflection_schema_is_unloaded() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state.world.expect_with_world_mut(|scene| {
        scene
            .set_dynamic_component(entity, CLOUD_LAYER_TYPE_PATH, json!({ "coverage": 0.25 }))
            .expect("legacy dynamic component should attach while no schema is registered");
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");

    assert!(!state
        .can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.coverage"))
        .expect("authoring world gateway should succeed"));
}
