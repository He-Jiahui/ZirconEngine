use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::plugin::ComponentTypeDescriptor;
use crate::scene::{components::NodeKind, World};
use serde_json::json;

#[test]
fn world_component_type_registry_gates_dynamic_component_attachment_when_present() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .register_component_type(
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property("coverage", "scalar", true),
        )
        .expect("register component type");

    let duplicate = world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap_err();
    assert_eq!(
        duplicate,
        "component type weather.Component.CloudLayer already registered"
    );
    assert_eq!(
        world
            .component_type_descriptor("weather.Component.CloudLayer")
            .expect("component descriptor")
            .display_name,
        "Cloud Layer"
    );
    assert_eq!(world.component_type_descriptors().len(), 1);

    let unknown = world
        .set_dynamic_component(entity, "lighting.Component.ProbeOverride", json!({}))
        .unwrap_err();
    assert_eq!(
        unknown,
        "dynamic component type `lighting.Component.ProbeOverride` is not registered"
    );
    assert!(world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.75 }),
        )
        .expect("attach registered dynamic component"));
}

#[test]
fn dynamic_plugin_components_attach_to_entities_and_roundtrip_with_world_serialization() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);

    assert!(world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({
                "coverage": 0.75,
                "label": "storm front",
                "enabled": true
            }),
        )
        .expect("attach dynamic component"));

    assert_eq!(
        world.dynamic_component(entity, "weather.Component.CloudLayer"),
        Some(&json!({
            "coverage": 0.75,
            "label": "storm front",
            "enabled": true
        }))
    );
    assert_eq!(
        world
            .property(
                entity,
                &ComponentPropertyPath::parse("weather.Component.CloudLayer.coverage").unwrap()
            )
            .unwrap(),
        ScenePropertyValue::Scalar(0.75)
    );

    let serialized = toml::to_string(&world).expect("serialize world");
    let restored: World = toml::from_str(&serialized).expect("deserialize world");
    assert_eq!(
        restored.dynamic_component(entity, "weather.Component.CloudLayer"),
        Some(&json!({
            "coverage": 0.75,
            "label": "storm front",
            "enabled": true
        }))
    );
}

#[test]
fn dynamic_plugin_component_property_writes_use_existing_scene_property_paths() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.25 }),
        )
        .unwrap();

    let changed = world
        .set_property(
            entity,
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.coverage").unwrap(),
            ScenePropertyValue::Scalar(0.9),
        )
        .unwrap();

    assert!(changed);
    assert_eq!(
        world.dynamic_component(entity, "weather.Component.CloudLayer"),
        Some(&json!({ "coverage": 0.9 }))
    );
}

#[test]
fn registered_dynamic_component_properties_gate_editor_writes() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .register_component_type(
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property("coverage", "scalar", true)
                .with_property("label", "string", false),
        )
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({
                "coverage": 0.25,
                "label": "storm front"
            }),
        )
        .unwrap();

    assert!(world
        .set_property(
            entity,
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.coverage").unwrap(),
            ScenePropertyValue::Scalar(0.9),
        )
        .unwrap());

    let readonly = world
        .set_property(
            entity,
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.label").unwrap(),
            ScenePropertyValue::String("cold front".to_string()),
        )
        .unwrap_err();
    assert_eq!(
        readonly,
        "dynamic component property `weather.Component.CloudLayer.label` is not editable"
    );

    let undeclared = world
        .set_property(
            entity,
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.density").unwrap(),
            ScenePropertyValue::Scalar(0.5),
        )
        .unwrap_err();
    assert_eq!(
        undeclared,
        "dynamic component type `weather.Component.CloudLayer` does not declare property `density`"
    );
}

#[test]
fn plugin_unload_is_blocked_while_entities_still_hold_plugin_components() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.25 }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            "lighting.Component.ProbeOverride",
            json!({ "enabled": true }),
        )
        .unwrap();

    let blocked = world
        .ensure_plugin_components_can_unload("weather")
        .unwrap_err();
    assert!(blocked.contains("weather.Component.CloudLayer"));
    assert!(blocked.contains(&format!("entity {entity}")));
    assert_eq!(world.dynamic_component_count_for_plugin("weather"), 1);

    world
        .remove_dynamic_component(entity, "weather.Component.CloudLayer")
        .unwrap();
    world
        .ensure_plugin_components_can_unload("weather")
        .unwrap();
    assert_eq!(
        world.dynamic_component(entity, "lighting.Component.ProbeOverride"),
        Some(&json!({ "enabled": true }))
    );
}
