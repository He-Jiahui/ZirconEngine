use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldValue, ReflectObjectAddress, ReflectReadRequest, ReflectTypeKind,
    ReflectWriteRequest, ReflectedValue,
};

use crate::plugin::ComponentTypeDescriptor;
use crate::scene::{NodeKind, World};

#[test]
fn dynamic_component_descriptor_registers_reflected_json_component() {
    let mut world = World::empty();
    let descriptor = cloud_layer_descriptor();

    world
        .register_component_type(descriptor.clone())
        .expect("dynamic descriptor should register");

    let registration = world
        .reflect_schema("weather.Component.CloudLayer")
        .expect("dynamic schema should be reflected");
    assert_eq!(registration.type_path.type_path, descriptor.type_id);
    assert_eq!(registration.type_path.short_type_path, "CloudLayer");
    assert_eq!(
        registration.type_path.plugin_id,
        Some("weather".to_string())
    );
    assert_eq!(registration.display_name, "Cloud Layer");
    assert_eq!(registration.type_info.kind, ReflectTypeKind::Json);
    assert!(registration.is_component);
    assert!(!registration.is_resource);
    assert!(registration.plugin_owned);
    assert!(registration.serializable);
    assert!(registration.editor_visible);
    assert!(registration.remote_visible);
    assert_eq!(registration.plugin_id, Some("weather".to_string()));
    assert_eq!(registration.type_info.fields.len(), 2);
    assert_eq!(registration.type_info.fields[0].name, "coverage");
    assert_eq!(registration.type_info.fields[0].display_name, "coverage");
    assert_eq!(registration.type_info.fields[0].value_type_path, "Scalar");
    assert!(registration.type_info.fields[0].editable);
    assert_eq!(registration.type_info.fields[1].name, "label");
    assert_eq!(registration.type_info.fields[1].display_name, "label");
    assert_eq!(registration.type_info.fields[1].value_type_path, "String");
    assert!(!registration.type_info.fields[1].editable);

    let entity = world.spawn_node(NodeKind::Mesh);
    let adapter = world
        .type_registry()
        .runtime_registration("CloudLayer")
        .expect("short dynamic type path should resolve")
        .component
        .clone()
        .expect("dynamic component registration should have adapter");
    assert!(!adapter.contains(&world, entity));
}

#[test]
fn dynamic_component_descriptor_duplicate_uses_reflection_preflight_error() {
    let mut world = World::empty();

    world
        .register_component_type(cloud_layer_descriptor())
        .expect("first dynamic descriptor should register");
    let duplicate = world
        .register_component_type(cloud_layer_descriptor())
        .expect_err("duplicate reflected type path should fail before descriptor mutation");

    assert_eq!(
        duplicate,
        ReflectError::DuplicateTypePath {
            type_path: "weather.Component.CloudLayer".to_string(),
        }
        .to_string()
    );
    assert_eq!(world.component_type_descriptors().len(), 1);
    assert_eq!(
        world
            .type_registry()
            .iter()
            .filter(|registration| registration.registration.type_path.type_path
                == "weather.Component.CloudLayer")
            .count(),
        1
    );
}

#[test]
fn dynamic_component_reflection_reads_json_property_through_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");
    let address = cloud_layer_address(entity);

    let read = world
        .reflect_read(ReflectReadRequest::new(address.clone(), "coverage"))
        .expect("dynamic field should read through reflection");
    assert_eq!(
        read.field,
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.75))
    );
    let fields = world
        .reflect_fields(zircon_runtime_interface::reflect::ReflectFieldsRequest::new(address))
        .expect("dynamic fields should enumerate in schema order")
        .fields;
    assert_eq!(
        fields,
        vec![
            ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.75)),
            ReflectFieldValue::new("label", ReflectedValue::String("storm front".to_string())),
        ]
    );
}

#[test]
fn dynamic_component_reflection_writes_json_property_through_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.25, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            cloud_layer_address(entity),
            "coverage",
            ReflectedValue::Scalar(0.9),
        ))
        .expect("editable dynamic field should write through reflection");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.9))
    );
    assert_eq!(
        world.dynamic_component(entity, "weather.Component.CloudLayer"),
        Some(&json!({ "coverage": 0.9, "label": "storm front" }))
    );
}

#[test]
fn dynamic_component_reflection_rejects_non_editable_property() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.25, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let error = world
        .reflect_write(ReflectWriteRequest::new(
            cloud_layer_address(entity),
            "label",
            ReflectedValue::String("cold front".to_string()),
        ))
        .expect_err("read-only dynamic field should be rejected");

    assert_eq!(
        error,
        ReflectError::NonEditableField {
            type_path: "weather.Component.CloudLayer".to_string(),
            field_name: "label".to_string(),
        }
    );
    assert_eq!(
        world.dynamic_component(entity, "weather.Component.CloudLayer"),
        Some(&json!({ "coverage": 0.25, "label": "storm front" }))
    );
}

#[test]
fn dynamic_component_reflection_unknown_type_and_field_are_structured_errors() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    let missing_component_entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::component(entity, "weather.Component.Unknown")
                    .expect("address should be valid"),
                "coverage",
            ))
            .expect_err("unknown reflected dynamic type should be structured"),
        ReflectError::UnknownType {
            type_path: "weather.Component.Unknown".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                cloud_layer_address(entity),
                "density"
            ))
            .expect_err("undeclared dynamic field should be structured"),
        ReflectError::UnknownField {
            type_path: "weather.Component.CloudLayer".to_string(),
            field_name: "density".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                cloud_layer_address(missing_component_entity),
                "coverage",
            ))
            .expect_err("missing dynamic component should be structured"),
        ReflectError::MissingComponent {
            entity: missing_component_entity,
            type_path: "weather.Component.CloudLayer".to_string(),
        }
    );
}

#[test]
fn plugin_unload_guard_still_counts_reflected_dynamic_components() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.25, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let blocked = world
        .ensure_plugin_components_can_unload("weather")
        .expect_err("plugin unload should still see dynamic component instances");

    assert!(blocked.contains("weather.Component.CloudLayer"));
    assert!(blocked.contains(&format!("entity {entity}")));
    assert_eq!(world.dynamic_component_count_for_plugin("weather"), 1);
    assert!(world
        .type_registry()
        .contains_type_path("weather.Component.CloudLayer"));
}

fn world_with_cloud_layer_descriptor() -> World {
    let mut world = World::empty();
    world
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    world
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

fn cloud_layer_address(entity: u64) -> ReflectObjectAddress {
    ReflectObjectAddress::component(entity, "weather.Component.CloudLayer")
        .expect("dynamic component address should be valid")
}
