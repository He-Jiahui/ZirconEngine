use std::collections::BTreeMap;

use crate::reflect::{
    ReflectEditorHint, ReflectEnumOption, ReflectError, ReflectFieldInfo, ReflectFieldValue,
    ReflectFieldsRequest, ReflectFieldsResponse, ReflectNumericRange, ReflectObjectAddress,
    ReflectReadRequest, ReflectReadResponse, ReflectSchemaFilter, ReflectSchemaRequest,
    ReflectSchemaResponse, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypeKind,
    ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectWriteResponse,
    ReflectedValue,
};

fn sample_registration() -> ReflectTypeRegistration {
    let fields = vec![
        ReflectFieldInfo::new("alpha", "f32", ReflectEditorHint::Scalar).with_numeric_range(
            ReflectNumericRange::new(Some(0.0), Some(1.0), Some(0.1), Some(2)),
        ),
        ReflectFieldInfo::new("name", "alloc::string::String", ReflectEditorHint::String)
            .with_default_value(ReflectedValue::String("default".to_string()))
            .with_documentation("Displayed sample name"),
    ];

    ReflectTypeRegistration::new(
        ReflectTypePath::new("zircon::Sample", "Sample").unwrap(),
        "Sample",
        ReflectTypeInfo::struct_with_fields(fields),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_remote_visible(true)
}

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn type_registration_serializes_with_ordered_fields() {
    let mut registration = sample_registration()
        .with_plugin_owned(true)
        .with_plugin_id("sample.plugin");
    registration.type_path = registration.type_path.with_module_path("zircon");
    let serialized = serde_json::to_value(&registration).unwrap();
    let fields = serialized
        .pointer("/type_info/fields")
        .and_then(serde_json::Value::as_array)
        .unwrap();

    assert_eq!(fields[0]["name"], "alpha");
    assert_eq!(fields[1]["name"], "name");
    assert_eq!(serialized["plugin_owned"], true);
    assert_eq!(serialized["serializable"], true);
    assert_eq!(serialized["editor_visible"], true);
    assert_eq!(serialized["remote_visible"], true);
    assert_eq!(serialized["plugin_id"], "sample.plugin");
    assert_eq!(serialized["type_path"]["plugin_id"], "sample.plugin");
    assert_eq!(registration.plugin_id.as_deref(), Some("sample.plugin"));
    assert_eq!(
        registration.type_path.plugin_id.as_deref(),
        Some("sample.plugin")
    );
    assert_eq!(
        round_trip::<ReflectTypeRegistration>(&registration),
        registration
    );
}

#[test]
fn reflected_value_tagged_json_roundtrips_all_supported_shapes() {
    let mut map = BTreeMap::new();
    map.insert("first".to_string(), ReflectedValue::Integer(-1));
    map.insert("second".to_string(), ReflectedValue::Unsigned(2));
    let values = vec![
        ReflectedValue::Bool(true),
        ReflectedValue::Integer(-2),
        ReflectedValue::Unsigned(2),
        ReflectedValue::Scalar(1.5),
        ReflectedValue::String("text".to_string()),
        ReflectedValue::Enum("Dynamic".to_string()),
        ReflectedValue::Vec2([1.0, 2.0]),
        ReflectedValue::Vec3([1.0, 2.0, 3.0]),
        ReflectedValue::Vec4([1.0, 2.0, 3.0, 4.0]),
        ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ReflectedValue::Entity(Some(9)),
        ReflectedValue::Entity(None),
        ReflectedValue::Resource("res://mesh/cube".to_string()),
        ReflectedValue::List(vec![ReflectedValue::Bool(false)]),
        ReflectedValue::Map(map),
        ReflectedValue::Json(serde_json::json!({ "nested": true })),
        ReflectedValue::Null,
    ];

    for value in values {
        let json = serde_json::to_value(&value).unwrap();
        assert!(json.get("kind").is_some(), "missing tag for {value:?}");
        assert_eq!(
            serde_json::from_value::<ReflectedValue>(json).unwrap(),
            value
        );
    }
}

#[test]
fn reflect_contract_dtos_use_expected_json_shapes() {
    let component = ReflectObjectAddress::component(42, "zircon::Name").unwrap();
    let resource = ReflectObjectAddress::resource("zircon::FrameCounter").unwrap();
    let field = ReflectFieldValue::new("translation", ReflectedValue::Vec3([1.0, 2.0, 3.0]));
    let updated_field =
        ReflectFieldValue::new("translation", ReflectedValue::Vec3([4.0, 5.0, 6.0]));

    assert_eq!(
        serde_json::to_value(ReflectedValue::Enum("Dynamic".to_string())).unwrap(),
        serde_json::json!({ "kind": "Enum", "value": "Dynamic" })
    );
    assert_eq!(
        serde_json::to_value(ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0])).unwrap(),
        serde_json::json!({ "kind": "Quaternion", "value": [0.0, 0.0, 0.0, 1.0] })
    );
    assert_eq!(
        serde_json::to_value(&component).unwrap(),
        serde_json::json!({ "kind": "Component", "entity": 42, "type_path": "zircon::Name" })
    );
    assert_eq!(
        serde_json::to_value(&resource).unwrap(),
        serde_json::json!({ "kind": "Resource", "type_path": "zircon::FrameCounter" })
    );
    assert_eq!(
        serde_json::to_value(ReflectFieldsRequest::new(component.clone())).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" }
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectFieldsResponse::new(
            component.clone(),
            vec![field.clone()],
        ))
        .unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "fields": [{
                "field_name": "translation",
                "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
            }]
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectReadRequest::new(component.clone(), "translation")).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field_name": "translation"
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectReadResponse::new(component.clone(), field)).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field": {
                "field_name": "translation",
                "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
            }
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectWriteRequest::new(
            component.clone(),
            "translation",
            ReflectedValue::Vec3([4.0, 5.0, 6.0]),
        ))
        .unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field_name": "translation",
            "value": { "kind": "Vec3", "value": [4.0, 5.0, 6.0] }
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectWriteResponse::new(component, updated_field, true)).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field": {
                "field_name": "translation",
                "value": { "kind": "Vec3", "value": [4.0, 5.0, 6.0] }
            },
            "changed": true
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectError::UnknownField {
            type_path: "zircon::Name".to_string(),
            field_name: "label".to_string(),
        })
        .unwrap(),
        serde_json::json!({
            "kind": "UnknownField",
            "type_path": "zircon::Name",
            "field_name": "label"
        })
    );
}

#[test]
fn field_metadata_preserves_editability_defaults_and_docs() {
    let field = ReflectFieldInfo::new("body_type", "RigidBodyType", ReflectEditorHint::Enum)
        .with_display_name("Body Type")
        .with_default_value(ReflectedValue::String("Dynamic".to_string()))
        .with_numeric_range(ReflectNumericRange::new(
            Some(0.0),
            Some(10.0),
            Some(0.5),
            Some(1),
        ))
        .with_enum_options(vec![
            ReflectEnumOption::new("Dynamic", "Dynamic Body"),
            ReflectEnumOption::new("Static", "Static Body").with_documentation("Does not move"),
        ])
        .with_editable(false)
        .with_serializable(true)
        .with_editor_visible(true)
        .with_documentation("Physics body type");
    let json = serde_json::to_value(&ReflectFieldInfo::new(
        "visible",
        "bool",
        ReflectEditorHint::Bool,
    ))
    .unwrap();
    let round_trip = round_trip::<ReflectFieldInfo>(&field);

    assert_eq!(round_trip.display_name, "Body Type");
    assert_eq!(round_trip.value_type_path, "RigidBodyType");
    assert_eq!(round_trip.editor_hint, ReflectEditorHint::Enum);
    assert_eq!(
        round_trip.default_value,
        Some(ReflectedValue::String("Dynamic".to_string()))
    );
    assert_eq!(
        round_trip.documentation.as_deref(),
        Some("Physics body type")
    );
    assert_eq!(round_trip.numeric_range.unwrap().step, Some(0.5));
    assert_eq!(
        round_trip.enum_options[1].documentation.as_deref(),
        Some("Does not move")
    );
    assert!(!round_trip.editable);
    assert!(json.get("default_value").is_none());
    assert!(json.get("documentation").is_none());
    assert!(json.get("enum_options").is_none());
}

#[test]
fn reflect_object_address_schema_and_read_write_dtos_roundtrip() {
    let component = ReflectObjectAddress::component(42, "zircon::Name").unwrap();
    let resource = ReflectObjectAddress::resource("zircon::FrameCounter").unwrap();
    let field = ReflectFieldValue::new("translation", ReflectedValue::Vec3([1.0, 2.0, 3.0]));
    let fields_request = ReflectFieldsRequest::new(component.clone());
    let fields_response = ReflectFieldsResponse::new(component.clone(), vec![field.clone()]);
    let read_request = ReflectReadRequest::new(component.clone(), "translation");
    let read_response = ReflectReadResponse::new(component.clone(), field.clone());
    let write_request = ReflectWriteRequest::new(
        component.clone(),
        "translation",
        ReflectedValue::Vec3([4.0, 5.0, 6.0]),
    );
    let write_response = ReflectWriteResponse::new(
        component.clone(),
        ReflectFieldValue::new("translation", ReflectedValue::Vec3([4.0, 5.0, 6.0])),
        true,
    );
    let schema_filter = ReflectSchemaFilter::remote_visible();
    let schema_request = ReflectSchemaRequest::new(schema_filter.clone());
    let schema_response = ReflectSchemaResponse::new(vec![sample_registration()]);

    assert!(matches!(component, ReflectObjectAddress::Component { .. }));
    assert_eq!(component.type_path(), "zircon::Name");
    assert_eq!(resource.type_path(), "zircon::FrameCounter");
    assert!(schema_filter.remote_visible);
    assert!(schema_filter.include_components);
    assert!(ReflectSchemaFilter::editor_visible().editor_visible);
    assert!(ReflectSchemaRequest::remote_visible().filter.remote_visible);
    assert!(ReflectSchemaRequest::editor_visible().filter.editor_visible);
    let for_type_filter = ReflectSchemaFilter::for_type("zircon::Name");
    assert_eq!(for_type_filter.type_path.as_deref(), Some("zircon::Name"));
    assert!(for_type_filter.include_components);
    assert!(for_type_filter.include_resources);
    assert!(!for_type_filter.editor_visible);
    assert!(!for_type_filter.remote_visible);
    assert!(!for_type_filter.include_plugin_owned);
    assert_eq!(
        ReflectSchemaRequest::for_type("zircon::Name")
            .filter
            .type_path
            .as_deref(),
        Some("zircon::Name")
    );
    assert!(
        ReflectSchemaRequest::for_type("zircon::Name")
            .filter
            .include_components
    );
    assert!(
        ReflectSchemaRequest::for_type("zircon::Name")
            .filter
            .include_resources
    );
    assert_eq!(round_trip::<ReflectObjectAddress>(&component), component);
    assert_eq!(round_trip::<ReflectObjectAddress>(&resource), resource);
    assert_eq!(
        round_trip::<ReflectFieldsRequest>(&fields_request),
        fields_request
    );
    assert_eq!(
        round_trip::<ReflectFieldsResponse>(&fields_response),
        fields_response
    );
    assert_eq!(
        round_trip::<ReflectReadRequest>(&read_request),
        read_request
    );
    assert_eq!(
        round_trip::<ReflectReadResponse>(&read_response),
        read_response
    );
    assert_eq!(
        round_trip::<ReflectWriteRequest>(&write_request),
        write_request
    );
    assert_eq!(
        round_trip::<ReflectWriteResponse>(&write_response),
        write_response
    );
    assert_eq!(
        round_trip::<ReflectSchemaRequest>(&schema_request),
        schema_request
    );
    assert_eq!(
        round_trip::<ReflectSchemaResponse>(&schema_response),
        schema_response
    );
}

#[test]
fn reflect_error_display_includes_type_field_and_entity_context() {
    let errors = vec![
        ReflectError::InvalidTypePath {
            type_path: " ".to_string(),
            reason: "type path must not be empty".to_string(),
        },
        ReflectError::AddressKindMismatch {
            expected: "Component".to_string(),
            actual: "Resource".to_string(),
        },
        ReflectError::MissingComponent {
            entity: 123,
            type_path: "zircon::Name".to_string(),
        },
        ReflectError::UnknownField {
            type_path: "zircon::Name".to_string(),
            field_name: "label".to_string(),
        },
        ReflectError::NoComponentAdapter {
            type_path: "zircon::Name".to_string(),
        },
        ReflectError::NoResourceAdapter {
            type_path: "zircon::FrameCounter".to_string(),
        },
        ReflectError::TypeMismatch {
            type_path: "zircon::Transform".to_string(),
            field_name: "translation".to_string(),
            expected: "Vec3".to_string(),
            actual: "String".to_string(),
        },
    ];
    let display = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(matches!(
        ReflectTypePath::new("", "Name"),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(matches!(
        ReflectTypePath::new("zircon::Name", " "),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(display.contains("type path"));
    assert!(display.contains("Component"));
    assert!(display.contains("Resource"));
    assert!(display.contains("123"));
    assert!(display.contains("zircon::Name"));
    assert!(display.contains("label"));
    assert!(display.contains("component adapter"));
    assert!(display.contains("resource adapter"));
    assert!(display.contains("zircon::Transform"));
    assert!(display.contains("translation"));
    assert!(display.contains("Vec3"));
    assert!(display.contains("String"));
}

#[test]
fn type_paths_and_type_kinds_use_approved_json_contract() {
    let path = ReflectTypePath::new("zircon::scene::Name", "Name").unwrap();
    let json = serde_json::to_value(&path).unwrap();
    let tuple_struct = serde_json::to_value(&ReflectTypeKind::TupleStruct).unwrap();
    let opaque = ReflectTypeInfo::opaque();
    let json_type = ReflectTypeInfo::json_with_fields(Vec::new());

    assert!(json.get("module_path").is_none());
    assert!(json.get("plugin_id").is_none());
    assert_eq!(tuple_struct, "tuple_struct");
    assert_eq!(opaque.kind, ReflectTypeKind::Opaque);
    assert_eq!(json_type.kind, ReflectTypeKind::Json);
}
