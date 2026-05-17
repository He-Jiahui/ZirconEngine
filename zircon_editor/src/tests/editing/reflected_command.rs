use serde_json::json;
use zircon_runtime::plugin::ComponentTypeDescriptor;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::reflect::{ReflectObjectAddress, ReflectReadRequest, ReflectedValue};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::intent::EditorIntent;

use super::support::{cube_and_camera, cube_id, test_state};

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const HIERARCHY_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
const LOCAL_TRANSFORM_TYPE_PATH: &str = "zircon_runtime::scene::components::LocalTransform";
const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const WIND_ANCHOR_TYPE_PATH: &str = "weather.Component.WindAnchor";

#[test]
fn reflected_editor_command_updates_fixed_component_and_undoes() {
    let mut scene = Scene::empty();
    let entity = scene.spawn_node(NodeKind::Cube);
    let original_name = scene.find_node(entity).unwrap().name.clone();

    let command = EditorCommand::set_reflected_scene_field(
        &mut scene,
        Some(entity),
        entity,
        NAME_TYPE_PATH,
        "value",
        ReflectedValue::String("Cloud".to_string()),
    )
    .expect("fixed reflected field command should be captured")
    .expect("name change should create a command");

    assert_eq!(scene.find_node(entity).unwrap().name, "Cloud");
    assert_eq!(command.target_node(), entity);
    assert_eq!(command.selection_after(), Some(entity));

    assert_eq!(command.undo(&mut scene).unwrap(), Some(entity));
    assert_eq!(scene.find_node(entity).unwrap().name, original_name);

    assert_eq!(command.apply(&mut scene).unwrap(), Some(entity));
    assert_eq!(scene.find_node(entity).unwrap().name, "Cloud");

    let no_op = EditorCommand::set_reflected_scene_field(
        &mut scene,
        Some(entity),
        entity,
        NAME_TYPE_PATH,
        "value",
        ReflectedValue::String("Cloud".to_string()),
    )
    .expect("same reflected value should be a valid no-op");
    assert!(no_op.is_none());
}

#[test]
fn reflected_editor_command_updates_dynamic_plugin_component_and_undoes() {
    let mut scene = scene_with_cloud_layer();
    let entity = scene.spawn_node(NodeKind::Mesh);
    scene
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "thin" }),
        )
        .expect("dynamic component should attach");

    let command = EditorCommand::set_reflected_scene_field(
        &mut scene,
        Some(entity),
        entity,
        CLOUD_LAYER_TYPE_PATH,
        "coverage",
        ReflectedValue::Scalar(0.75),
    )
    .expect("dynamic reflected field command should be captured")
    .expect("coverage change should create a command");

    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "coverage"),
        ReflectedValue::Scalar(0.75)
    );

    assert_eq!(command.undo(&mut scene).unwrap(), Some(entity));
    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "coverage"),
        ReflectedValue::Scalar(0.25)
    );

    assert_eq!(command.apply(&mut scene).unwrap(), Some(entity));
    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "coverage"),
        ReflectedValue::Scalar(0.75)
    );
}

#[test]
fn reflected_editor_command_rejects_readonly_dynamic_field() {
    let mut scene = scene_with_cloud_layer();
    let entity = scene.spawn_node(NodeKind::Mesh);
    scene
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "thin" }),
        )
        .expect("dynamic component should attach");

    let error = EditorCommand::set_reflected_scene_field(
        &mut scene,
        Some(entity),
        entity,
        CLOUD_LAYER_TYPE_PATH,
        "label",
        ReflectedValue::String("storm".to_string()),
    )
    .expect_err("read-only reflected dynamic fields must be rejected");

    assert!(error.contains("not editable"));
    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "label"),
        ReflectedValue::String("thin".to_string())
    );
}

#[test]
fn reflected_editor_command_uses_reflection_schema_for_dynamic_field_editability() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state.world.with_world_mut(|scene| {
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

    assert!(state.can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.coverage")));
    assert!(!state.can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.label")));
    assert!(!state.can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.missing")));
}

#[test]
fn reflected_editor_command_rejects_dynamic_field_when_reflection_schema_is_unloaded() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state.world.with_world_mut(|scene| {
        scene
            .set_dynamic_component(entity, CLOUD_LAYER_TYPE_PATH, json!({ "coverage": 0.25 }))
            .expect("legacy dynamic component should attach while no schema is registered");
    });
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .expect("selection should succeed");

    assert!(!state.can_edit_dynamic_component_field(&format!("{CLOUD_LAYER_TYPE_PATH}.coverage")));
}

#[test]
fn reflected_editor_command_snapshot_uses_reflection_schema_for_plugin_properties() {
    let mut state = test_state();
    let entity = cube_id(&state);
    state.world.with_world_mut(|scene| {
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
    state.world.with_world_mut(|scene| {
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
    state.world.with_world_mut(|scene| {
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

#[test]
fn reflected_editor_command_routes_inspector_fields_through_reflection() {
    let mut state = test_state();
    let (entity, camera) = cube_and_camera(&state);
    let original_name = state
        .world
        .with_world(|scene| read_reflected_field(scene, entity, NAME_TYPE_PATH, "value"));
    let original_parent = state
        .world
        .with_world(|scene| read_reflected_field(scene, entity, HIERARCHY_TYPE_PATH, "parent"));
    let original_translation = state.world.with_world(|scene| {
        read_reflected_field(scene, entity, LOCAL_TRANSFORM_TYPE_PATH, "translation")
    });
    state.world.with_world_mut(|scene| {
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
    state.inspector_dynamic_fields.insert(
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage"),
        "0.8".to_string(),
    );

    assert!(state
        .apply_inspector_changes()
        .expect("inspector change should apply through reflected command"));
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        ReflectedValue::String("Reflected Cube".to_string())
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        ReflectedValue::Vec3([3.5, 4.5, 5.5])
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.8)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        original_name
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        original_parent
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        original_translation
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.25)
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value"
        )),
        ReflectedValue::String("Reflected Cube".to_string())
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "translation"
        )),
        ReflectedValue::Vec3([3.5, 4.5, 5.5])
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
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
    let original_parent = state
        .world
        .with_world(|scene| read_reflected_field(scene, entity, HIERARCHY_TYPE_PATH, "parent"));
    let original_scale = state.world.with_world(|scene| {
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
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        ReflectedValue::Vec3([2.0, 3.0, 4.0])
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        ReflectedValue::Entity(Some(camera))
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        original_scale
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            HIERARCHY_TYPE_PATH,
            "parent"
        )),
        original_parent
    );

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        ReflectedValue::Vec3([2.0, 3.0, 4.0])
    );
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
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
    let original_scale = state.world.with_world(|scene| {
        read_reflected_field(scene, entity, LOCAL_TRANSFORM_TYPE_PATH, "scale")
    });

    state.update_dynamic_component_field(
        format!("{LOCAL_TRANSFORM_TYPE_PATH}.scale"),
        "1.0, nope, 3.0".to_string(),
    );

    let error = state
        .apply_inspector_changes()
        .expect_err("invalid vector text should be rejected before mutation");
    assert!(error.contains("Vec3"));
    assert_eq!(
        state.world.with_world(|scene| read_reflected_field(
            scene,
            entity,
            LOCAL_TRANSFORM_TYPE_PATH,
            "scale"
        )),
        original_scale
    );
}

fn scene_with_cloud_layer() -> Scene {
    let mut scene = Scene::empty();
    scene
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic component descriptor should register");
    scene
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

fn wind_anchor_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(WIND_ANCHOR_TYPE_PATH, "weather", "Wind Anchor")
        .with_property("direction", "Vec3", true)
        .with_property("target", "Entity", true)
}

fn read_reflected_field(
    scene: &Scene,
    entity: NodeId,
    type_path: &str,
    field_name: &str,
) -> ReflectedValue {
    scene
        .reflect_read(ReflectReadRequest::new(
            ReflectObjectAddress::component(entity, type_path)
                .expect("test component address should be valid"),
            field_name,
        ))
        .expect("reflected field should be readable")
        .field
        .value
}
