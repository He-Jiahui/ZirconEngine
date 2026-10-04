use serde_json::json;

use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_and_camera, cube_id, test_state};

use super::support::{
    cloud_layer_descriptor, wind_anchor_descriptor, CLOUD_LAYER_TYPE_PATH, WIND_ANCHOR_TYPE_PATH,
};

#[test]
fn reflected_editor_command_snapshot_uses_reflection_schema_for_plugin_properties() {
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
    state.update_dynamic_component_field(format!("{CLOUD_LAYER_TYPE_PATH}.coverage"), "0.6".into());

    let snapshot = state.snapshot();
    let plugin_component = snapshot
        .inspector
        .expect("selected node should project inspector")
        .plugin_components
        .into_iter()
        .find(|component| component.component_id == CLOUD_LAYER_TYPE_PATH)
        .expect("cloud layer component should be projected");

    assert_eq!(plugin_component.display_name, "Cloud Layer");
    assert_eq!(plugin_component.plugin_id, "weather");
    assert_eq!(plugin_component.properties.len(), 2);

    let coverage = plugin_component
        .properties
        .iter()
        .find(|property| property.name == "coverage")
        .expect("coverage property should be reflected");
    assert_eq!(
        coverage.field_id,
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage")
    );
    assert_eq!(coverage.label, "Coverage");
    assert_eq!(coverage.value, "0.6");
    assert_eq!(coverage.value_kind, "Scalar");
    assert!(coverage.editable);

    let label = plugin_component
        .properties
        .iter()
        .find(|property| property.name == "label")
        .expect("label property should be reflected");
    assert_eq!(label.value, "thin");
    assert_eq!(label.value_kind, "String");
    assert!(!label.editable);
}

#[test]
fn reflected_editor_command_snapshot_keeps_unloaded_dynamic_schema_protected() {
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

    let snapshot = state.snapshot();
    let plugin_component = snapshot
        .inspector
        .expect("selected node should project inspector")
        .plugin_components
        .into_iter()
        .find(|component| component.component_id == CLOUD_LAYER_TYPE_PATH)
        .expect("unloaded dynamic component should still be visible");

    assert!(plugin_component
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("serialized data stays protected")));
    let coverage = plugin_component
        .properties
        .iter()
        .find(|property| property.name == "coverage")
        .expect("serialized JSON property should be shown");
    assert_eq!(coverage.value, "0.25");
    assert!(!coverage.editable);
}

#[test]
fn reflected_editor_command_snapshot_marks_vector_and_entity_fields_editable() {
    let mut state = test_state();
    let (entity, camera) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene
            .register_component_type(wind_anchor_descriptor())
            .expect("dynamic component descriptor should register");
        scene
            .set_dynamic_component(
                entity,
                WIND_ANCHOR_TYPE_PATH,
                json!({
                    "direction": [1.0, 0.0, 0.5],
                    "target": { "entity": camera }
                }),
            )
            .expect("dynamic component should attach");
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");
    state.update_dynamic_component_field(
        format!("{WIND_ANCHOR_TYPE_PATH}.direction"),
        "2.0, 3.0, 4.0".to_string(),
    );

    let snapshot = state.snapshot();
    let plugin_component = snapshot
        .inspector
        .expect("selected node should project inspector")
        .plugin_components
        .into_iter()
        .find(|component| component.component_id == WIND_ANCHOR_TYPE_PATH)
        .expect("wind anchor component should be projected");

    let direction = plugin_component
        .properties
        .iter()
        .find(|property| property.name == "direction")
        .expect("direction property should be reflected");
    assert_eq!(direction.value, "2.0, 3.0, 4.0");
    assert_eq!(direction.value_kind, "Vec3");
    assert!(direction.editable);

    let target = plugin_component
        .properties
        .iter()
        .find(|property| property.name == "target")
        .expect("target property should be reflected");
    assert_eq!(target.value, camera.to_string());
    assert_eq!(target.value_kind, "Entity");
    assert!(target.editable);
}
