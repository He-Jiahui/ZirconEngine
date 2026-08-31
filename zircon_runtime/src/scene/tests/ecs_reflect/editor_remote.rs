use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldId, ReflectFieldValue, ReflectFieldsRequest, ReflectObjectAddress,
    ReflectReadRequest, ReflectReadResponse, ReflectSchemaRequest, ReflectSchemaResponse,
    ReflectWriteRequest, ReflectWriteResponse, ReflectedValue,
};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::reflect::RUNTIME_REFLECT_VALUE_BUDGET;
use crate::scene::{components::ActiveSelf, components::Name, NodeKind, World};

#[test]
fn inspector_style_field_list_uses_world_reflection_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .insert(entity, Name("Inspector Mesh".to_string()))
        .expect("name setup should use normal world mutation");
    world
        .insert(entity, ActiveSelf(false))
        .expect("active setup should use normal world mutation");
    world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.35, "label": "inspector cloud" }),
        )
        .expect("dynamic component should attach");

    let name_fields = world
        .reflect_fields(ReflectFieldsRequest::new(component_address(entity, "Name")))
        .expect("inspector should list Name through reflection")
        .fields;
    assert_eq!(
        name_fields,
        vec![ReflectFieldValue::new(
            ReflectFieldId::from_stable_keys("zircon_runtime::scene::components::Name", "value"),
            "value",
            ReflectedValue::String("Inspector Mesh".to_string()),
        )]
    );

    let active_fields = world
        .reflect_fields(ReflectFieldsRequest::new(component_address(
            entity,
            "ActiveSelf",
        )))
        .expect("inspector should list ActiveSelf through reflection")
        .fields;
    assert_eq!(
        active_fields,
        vec![ReflectFieldValue::new(
            ReflectFieldId::from_stable_keys(
                "zircon_runtime::scene::components::ActiveSelf",
                "value",
            ),
            "value",
            ReflectedValue::Bool(false),
        )]
    );

    let dynamic_fields = world
        .reflect_fields(ReflectFieldsRequest::new(cloud_layer_address(entity)))
        .expect("inspector should list dynamic component through reflection")
        .fields;
    assert_eq!(
        dynamic_fields,
        vec![
            ReflectFieldValue::new(
                cloud_layer_field_id("coverage"),
                "coverage",
                ReflectedValue::Scalar(0.35),
            ),
            ReflectFieldValue::new(
                cloud_layer_field_id("label"),
                "label",
                ReflectedValue::String("inspector cloud".to_string()),
            ),
        ]
    );
}

#[test]
fn remote_style_schema_read_request_response_serializes_without_runtime_handles() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.65, "label": "remote cloud" }),
        )
        .expect("dynamic component should attach");

    let default_request = roundtrip_dto(
        &ReflectSchemaRequest::remote_visible(),
        "schema request should serialize",
        "schema request should deserialize",
    );
    let default_schema = world
        .list_reflect_types(default_request)
        .expect("remote schema should apply plugin ownership defaults");
    assert!(
        !contains_cloud_layer(&default_schema),
        "plugin-owned dynamic types require explicit remote schema opt-in"
    );

    let mut request = ReflectSchemaRequest::remote_visible();
    request.filter.include_plugin_owned = true;
    let request = roundtrip_dto(
        &request,
        "schema request with plugin opt-in should serialize",
        "schema request with plugin opt-in should deserialize",
    );
    let schema = world
        .list_reflect_types(request)
        .expect("remote schema should list reflected plugin components");
    assert!(contains_cloud_layer(&schema));

    let schema_json = serde_json::to_string(&schema).expect("schema DTO should serialize");
    assert_runtime_handles_absent(&schema_json);
    let decoded_schema: ReflectSchemaResponse =
        serde_json::from_str(&schema_json).expect("schema DTO should deserialize");
    assert_eq!(decoded_schema, schema);

    let read_request = roundtrip_dto(
        &ReflectReadRequest::new(
            cloud_layer_address(entity),
            cloud_layer_field_id("coverage"),
        ),
        "read request should serialize",
        "read request should deserialize",
    );
    assert_runtime_handles_absent(
        &serde_json::to_string(&read_request)
            .expect("read request should serialize for leak check"),
    );

    let read = world
        .reflect_read(read_request)
        .expect("remote read should use reflection facade");
    assert_eq!(
        read.field,
        ReflectFieldValue::new(
            cloud_layer_field_id("coverage"),
            "coverage",
            ReflectedValue::Scalar(0.65),
        )
    );

    let read_json = serde_json::to_string(&read).expect("read DTO should serialize");
    assert_runtime_handles_absent(&read_json);
    let decoded_read: ReflectReadResponse =
        serde_json::from_str(&read_json).expect("read DTO should deserialize");
    assert_eq!(decoded_read, read);
}

#[test]
fn schema_list_projection_pre_sizes_registration_output() {
    let source = include_str!("../../reflect/world_reflection.rs");
    let list_reflect_types = source
        .split("pub fn list_reflect_types")
        .nth(1)
        .and_then(|text| text.split("pub fn reflect_schema").next())
        .expect("read WorldReflection list_reflect_types body");

    assert!(
        list_reflect_types.contains("let mut registrations = Vec::with_capacity(1);")
            && list_reflect_types.contains("registrations.push(registration.clone());")
            && list_reflect_types.contains("let registry_entries = schema_catalog.registrations();")
            && list_reflect_types.contains("Vec::with_capacity(registry_entries.size_hint().0)")
            && list_reflect_types.contains("for registration in registry_entries")
            && list_reflect_types.contains("if schema_filter_matches(registration, &filter)")
            && list_reflect_types.contains("schema_catalog.fingerprint()")
            && !list_reflect_types.contains("runtime_registration(type_path)")
            && !list_reflect_types.contains(".collect()"),
        "WorldReflection schema listing must project one catalog snapshot/fingerprint and pre-size filtered output"
    );
}

#[test]
fn component_adapter_lookup_borrows_for_read_paths_and_clones_only_for_write() {
    let source = include_str!("../../reflect/world_reflection.rs");
    let adapter_lookup = source
        .split("fn component_adapter<'a>")
        .nth(1)
        .and_then(|text| text.split("fn resource_adapter_ref").next())
        .expect("read WorldReflection component adapter lookup helpers");
    let resource_lookup = source
        .split("fn resource_adapter_ref<'a>")
        .nth(1)
        .and_then(|text| text.split("fn resource_adapter_for_write").next())
        .expect("read WorldReflection resource adapter lookup helper");
    let reflect_fields = source
        .split("pub fn reflect_fields")
        .nth(1)
        .and_then(|text| text.split("pub fn reflect_read").next())
        .expect("read WorldReflection reflect_fields body");
    let read_reflected_field = source
        .split("fn read_reflected_field")
        .nth(1)
        .expect("read WorldReflection read helper body");
    let reflect_write = source
        .split("pub fn reflect_write")
        .nth(1)
        .and_then(|text| text.split("impl World").next())
        .expect("read WorldReflection reflect_write body");

    assert!(
        adapter_lookup.contains("Result<&'a ReflectComponent, ReflectError>")
            && adapter_lookup.contains("let Some(adapter) = registration.component.as_ref() else")
            && adapter_lookup.contains("return Err(ReflectError::NoComponentAdapter")
            && adapter_lookup.contains("Ok(adapter)")
            && adapter_lookup.contains("fn component_adapter_for_write(")
            && adapter_lookup.contains("component_adapter(world, type_path).cloned()")
            && resource_lookup.contains("let Some(adapter) = registration.resource.as_ref() else")
            && resource_lookup.contains("return Err(ReflectError::NoResourceAdapter")
            && resource_lookup.contains("Ok(adapter)")
            && source.contains("fn resource_adapter_for_write(")
            && source.contains("resource_adapter_ref(world, type_path).copied()")
            && reflect_fields.contains("let adapter = component_adapter(world, type_path)?;")
            && read_reflected_field.contains("let adapter = component_adapter(world, type_path)?;")
            && reflect_write
                .contains("let adapter = component_adapter_for_write(world, type_path)?;")
            && reflect_write
                .contains("let adapter = resource_adapter_for_write(world, type_path)?;")
            && !adapter_lookup.contains(".component\n        .clone()")
            && !adapter_lookup.contains(".ok_or_else(|| ReflectError::NoComponentAdapter")
            && !resource_lookup.contains(".ok_or_else(|| ReflectError::NoResourceAdapter"),
        "WorldReflection must borrow component/resource adapters through direct success-path branches and reserve component adapter cloning for write paths that need to release the immutable registry borrow"
    );
}

#[test]
fn reflection_write_returns_the_accepted_request_without_post_write_readback() {
    let source = include_str!("../../reflect/world_reflection.rs");
    let reflect_write = source
        .split("pub fn reflect_write")
        .nth(1)
        .and_then(|text| text.split("impl World").next())
        .expect("read WorldReflection reflect_write body");

    assert!(
        reflect_write
            .contains("validate_reflected_value(type_path, &field_name, &request.value)?;")
            && reflect_write.contains("let accepted_field =")
            && reflect_write
                .contains("field_access(world, type_path, request.field_id, true)?;")
            && reflect_write.contains("adapter.write_field_by_slot(")
            && !reflect_write.contains("request.field_name")
            && !reflect_write.contains(".find(|field| field.name ==")
            && !reflect_write.contains("adapter.write_field(")
            && !reflect_write.contains("adapter.read_field(world, *entity, &request.field_name)?")
            && !reflect_write.contains("adapter.read_field(world, &request.field_name)?")
            && !reflect_write.contains("read_reflected_field(world, &request.address"),
        "reflection writes must return the accepted request field after publication without invoking a second read adapter"
    );
}

#[test]
fn remote_style_write_request_serializes_and_mutates_through_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "remote cloud" }),
        )
        .expect("dynamic component should attach");
    let address = cloud_layer_address(entity);

    let write = ReflectWriteRequest::new(
        address.clone(),
        cloud_layer_field_id("coverage"),
        ReflectedValue::Scalar(0.8),
    );
    let decoded_write = roundtrip_dto(
        &write,
        "write request should serialize",
        "write request should deserialize",
    );
    assert_runtime_handles_absent(
        &serde_json::to_string(&decoded_write)
            .expect("write request should serialize for leak check"),
    );

    let response = world
        .reflect_write(decoded_write)
        .expect("remote write should mutate through reflection facade");
    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new(
            cloud_layer_field_id("coverage"),
            "coverage",
            ReflectedValue::Scalar(0.8),
        )
    );
    let response_json = serde_json::to_string(&response).expect("write response should serialize");
    assert_runtime_handles_absent(&response_json);
    let decoded_response: ReflectWriteResponse =
        serde_json::from_str(&response_json).expect("write response should deserialize");
    assert_eq!(decoded_response, response);

    let read_back = world
        .reflect_read(ReflectReadRequest::new(
            address,
            cloud_layer_field_id("coverage"),
        ))
        .expect("readback should observe the reflected write");
    assert_eq!(
        read_back.field,
        ReflectFieldValue::new(
            cloud_layer_field_id("coverage"),
            "coverage",
            ReflectedValue::Scalar(0.8),
        )
    );
}

#[test]
fn reflection_value_budget_rejects_inbound_before_mutation_and_outbound_before_publication() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "stable" }),
        )
        .expect("dynamic component should attach");
    let address = cloud_layer_address(entity);
    let error = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            cloud_layer_field_id("coverage"),
            ReflectedValue::Scalar(f32::NAN),
        ))
        .expect_err("non-finite editor write must fail before mutation");
    assert!(matches!(
        error,
        ReflectError::InvalidValue {
            ref type_path,
            ref field_name,
            ..
        } if type_path == CLOUD_LAYER_TYPE_PATH && field_name == "coverage"
    ));
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                address.clone(),
                cloud_layer_field_id("coverage"),
            ))
            .expect("rejected write must preserve the old value")
            .field
            .value,
        ReflectedValue::Scalar(0.25)
    );

    let over_budget = "x".repeat(RUNTIME_REFLECT_VALUE_BUDGET.max_string_bytes() + 1);
    let error = world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": over_budget }),
        )
        .expect_err("dynamic component admission must reject over-budget field values");
    assert!(matches!(
        error,
        crate::scene::SceneError::Reflect(ReflectError::InvalidValue { .. })
    ));
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                address,
                cloud_layer_field_id("label"),
            ))
            .expect("rejected dynamic payload must preserve the old value")
            .field
            .value,
        ReflectedValue::String("stable".to_string())
    );
}

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";

fn world_with_cloud_layer_descriptor() -> World {
    let mut world = World::empty();
    world
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    world
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

fn component_address(entity: u64, type_path: &str) -> ReflectObjectAddress {
    ReflectObjectAddress::component(entity, type_path).expect("component address should be valid")
}

fn cloud_layer_address(entity: u64) -> ReflectObjectAddress {
    component_address(entity, CLOUD_LAYER_TYPE_PATH)
}

fn cloud_layer_field_id(field_key: &str) -> ReflectFieldId {
    ReflectFieldId::from_stable_keys(CLOUD_LAYER_TYPE_PATH, field_key)
}

fn contains_cloud_layer(schema: &ReflectSchemaResponse) -> bool {
    schema
        .registrations
        .iter()
        .any(|registration| registration.type_path.type_path() == CLOUD_LAYER_TYPE_PATH)
}

fn roundtrip_dto<T>(value: &T, serialize_context: &str, deserialize_context: &str) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let json = serde_json::to_string(value).expect(serialize_context);
    assert_runtime_handles_absent(&json);
    serde_json::from_str(&json).expect(deserialize_context)
}

fn assert_runtime_handles_absent(json: &str) {
    for runtime_only in [
        "type_registry",
        "RuntimeTypeRegistration",
        "ReflectComponent",
        "ReflectResource",
        "World",
    ] {
        assert!(
            !json.contains(runtime_only),
            "serialized reflection DTO leaked runtime-only token `{runtime_only}`: {json}"
        );
    }
}
