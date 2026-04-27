use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::{components::NodeKind, World};
use serde_json::json;

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
