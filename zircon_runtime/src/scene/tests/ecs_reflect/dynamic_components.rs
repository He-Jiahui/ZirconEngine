use serde_json::json;
use std::path::{Path, PathBuf};
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
fn dynamic_component_reflection_registration_pre_sizes_descriptor_fields() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let registration = source
        .split("pub fn registration_from_component_descriptor")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn reflect_component_for_dynamic_descriptor")
                .next()
        })
        .expect("read dynamic component reflection registration body");

    assert!(
        registration.contains("let mut fields = Vec::with_capacity(descriptor.properties.len());")
            && registration.contains("for property in &descriptor.properties")
            && registration.contains("fields.push(field_from_property_descriptor(")
            && registration.contains("&descriptor.type_id")
            && registration.contains("property")
            && !registration.contains(".collect::<Result<Vec<_>, _>>()")
            && !registration.contains(".map(|property| field_from_property_descriptor"),
        "dynamic component reflection registration must pre-size the reflected field list from descriptor properties and push fields directly"
    );
}

#[test]
fn dynamic_component_reflection_field_info_uses_constructor_display_name() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let field_builder = source
        .split("fn field_from_property_descriptor")
        .nth(1)
        .and_then(|text| text.split("fn short_type_path").next())
        .expect("read dynamic component field descriptor projection body");

    assert!(
        field_builder.contains("ReflectFieldInfo::new(")
            && field_builder.contains("descriptor.name.clone(),")
            && field_builder.contains("descriptor.value_type.clone(),")
            && field_builder.contains("ReflectEditorHint::None")
            && field_builder.contains(".with_editable(descriptor.editable)")
            && !field_builder.contains(".with_display_name("),
        "dynamic component reflection fields must rely on ReflectFieldInfo::new for same-as-name display labels instead of cloning and overwriting the display name"
    );
}

#[test]
fn dynamic_component_reflection_short_type_path_uses_direct_split_branch() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let short_type_path = source
        .split("fn short_type_path")
        .nth(1)
        .and_then(|text| text.split("fn contains").next())
        .expect("read dynamic component short type-path helper");

    assert!(
        short_type_path.contains("if let Some((_, short)) = type_path.rsplit_once('.')")
            && short_type_path.contains("return short;")
            && short_type_path.contains("type_path")
            && !short_type_path.contains(".map(|(_, short)| short)")
            && !short_type_path.contains(".unwrap_or(type_path)"),
        "dynamic component short type-path parsing must use a direct split branch instead of Option adapter chaining"
    );
}

#[test]
fn dynamic_component_reflection_read_fields_pre_sizes_result_vector() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let read_fields = source
        .split("fn read_fields")
        .nth(1)
        .and_then(|text| text.split("fn write_field").next())
        .expect("read dynamic component reflection read_fields body");

    assert!(
        read_fields.contains("let fields = &registration.type_info.fields;")
            && read_fields.contains("let mut values = Vec::with_capacity(fields.len());")
            && read_fields.contains("for field in fields")
            && read_fields.contains("let value = read_field(world, entity, type_path, &field.name)?;")
            && read_fields.contains("values.push(ReflectFieldValue::new(field.name.clone(), value));")
            && read_fields.contains("Ok(values)")
            && !read_fields.contains(".collect()")
            && !read_fields.contains(".map(|field|"),
        "dynamic component reflection read_fields must pre-size reflected field results and push directly instead of relying on collect growth"
    );
}

#[test]
fn dynamic_component_reflection_property_path_uses_direct_constructor() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let property_path = source
        .split("fn dynamic_property_path")
        .nth(1)
        .expect("read dynamic component reflection property-path helper");

    assert!(
        property_path.contains("let mut property_segments = Vec::with_capacity(1);")
            && property_path.contains("property_segments.push(field_name.to_string());")
            && property_path
                .contains("ComponentPropertyPath::new(type_path.to_string(), property_segments)")
            && !property_path.contains("ComponentPropertyPath::parse(&path)")
            && !property_path
                .contains("String::with_capacity(type_path.len() + 1 + field_name.len())")
            && !property_path.contains("format!(\"{type_path}.{field_name}\")"),
        "dynamic component reflection property-path helper must construct the full component path directly instead of re-parsing a joined string"
    );
}

#[test]
fn dynamic_component_reflection_read_helpers_use_direct_success_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("reflect")
            .join("dynamic_component.rs"),
    );
    let read_field = source
        .split("fn read_field")
        .nth(1)
        .and_then(|text| text.split("fn read_fields").next())
        .expect("read dynamic component reflection read_field body");
    let ensure_dynamic_component = source
        .split("fn ensure_dynamic_component")
        .nth(1)
        .and_then(|text| text.split("fn ensure_declared_field").next())
        .expect("read dynamic component presence helper");
    let ensure_declared_field = source
        .split("fn ensure_declared_field")
        .nth(1)
        .and_then(|text| text.split("fn ensure_json_field_present").next())
        .expect("read reflected field declaration helper");
    let ensure_json_field_present = source
        .split("fn ensure_json_field_present")
        .nth(1)
        .and_then(|text| text.split("fn dynamic_property_path").next())
        .expect("read dynamic JSON field presence helper");

    assert!(
        read_field.contains(
            "let Some(value) = world.dynamic_component_property(entity, &property_path) else"
        )
            && read_field.contains("return Err(ReflectError::UnsupportedConversion")
            && ensure_dynamic_component
                .contains("let Some(component) = world.dynamic_component(entity, type_path) else")
            && ensure_dynamic_component.contains("return Err(ReflectError::MissingComponent")
            && ensure_dynamic_component.contains("Ok(component)")
            && ensure_declared_field.contains("for field in &registration.type_info.fields")
            && ensure_declared_field.contains("return Ok(field);")
            && ensure_declared_field.contains("Err(ReflectError::UnknownField")
            && ensure_json_field_present.contains("let Some(object) = component.as_object() else")
            && ensure_json_field_present.contains("if object.contains_key(field_name)")
            && ensure_json_field_present.contains("return Ok(());")
            && !read_field.contains(".ok_or_else(|| ReflectError::UnsupportedConversion")
            && !ensure_dynamic_component.contains(".ok_or_else(|| ReflectError::MissingComponent")
            && !ensure_declared_field.contains(".iter()\n        .find(")
            && !ensure_json_field_present.contains(".and_then(|object| object.get(field_name))")
            && !ensure_json_field_present.contains(".ok_or_else(|| ReflectError::UnknownField"),
        "dynamic component reflection read helpers must use direct success-path branches instead of closure-based Option conversion"
    );
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

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read source file {}: {error}", path.display()))
}
