use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectFieldValue, ReflectFieldsRequest, ReflectObjectAddress, ReflectReadRequest,
    ReflectReadResponse, ReflectSchemaRequest, ReflectSchemaResponse, ReflectWriteRequest,
    ReflectWriteResponse, ReflectedValue,
};

use crate::plugin::ComponentTypeDescriptor;
use crate::scene::{components::ActiveSelf, components::Name, NodeKind, World};

#[test]
fn inspector_style_field_list_uses_world_reflection_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
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
        vec![ReflectFieldValue::new("value", ReflectedValue::Bool(false))]
    );

    let dynamic_fields = world
        .reflect_fields(ReflectFieldsRequest::new(cloud_layer_address(entity)))
        .expect("inspector should list dynamic component through reflection")
        .fields;
    assert_eq!(
        dynamic_fields,
        vec![
            ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.35)),
            ReflectFieldValue::new(
                "label",
                ReflectedValue::String("inspector cloud".to_string()),
            ),
        ]
    );
}

#[test]
fn remote_style_schema_read_request_response_serializes_without_runtime_handles() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
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
        &ReflectReadRequest::new(cloud_layer_address(entity), "coverage"),
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
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.65))
    );

    let read_json = serde_json::to_string(&read).expect("read DTO should serialize");
    assert_runtime_handles_absent(&read_json);
    let decoded_read: ReflectReadResponse =
        serde_json::from_str(&read_json).expect("read DTO should deserialize");
    assert_eq!(decoded_read, read);
}

#[test]
fn remote_style_write_request_serializes_and_mutates_through_facade() {
    let mut world = world_with_cloud_layer_descriptor();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "remote cloud" }),
        )
        .expect("dynamic component should attach");
    let address = cloud_layer_address(entity);

    let write = ReflectWriteRequest::new(address.clone(), "coverage", ReflectedValue::Scalar(0.8));
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
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.8))
    );
    let response_json = serde_json::to_string(&response).expect("write response should serialize");
    assert_runtime_handles_absent(&response_json);
    let decoded_response: ReflectWriteResponse =
        serde_json::from_str(&response_json).expect("write response should deserialize");
    assert_eq!(decoded_response, response);

    let read_back = world
        .reflect_read(ReflectReadRequest::new(address, "coverage"))
        .expect("readback should observe the reflected write");
    assert_eq!(
        read_back.field,
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.8))
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

fn contains_cloud_layer(schema: &ReflectSchemaResponse) -> bool {
    schema
        .registrations
        .iter()
        .any(|registration| registration.type_path.type_path == CLOUD_LAYER_TYPE_PATH)
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
