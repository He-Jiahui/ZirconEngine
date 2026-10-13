//! 反射接口的序列化形状、规范化入口与错误上下文契约测试；不执行 Runtime 对象访问或注册流程。
use std::collections::BTreeMap;

use crate::reflect::{
    ReflectEditorHint, ReflectEnumOption, ReflectError, ReflectFieldId, ReflectFieldIdParseError,
    ReflectFieldInfo, ReflectFieldValue, ReflectFieldsRequest, ReflectFieldsResponse,
    ReflectNumericRange, ReflectNumericRangeError, ReflectObjectAddress, ReflectReadRequest,
    ReflectReadResponse, ReflectSchemaCatalog, ReflectSchemaCatalogEntry, ReflectSchemaFilter,
    ReflectSchemaRequest, ReflectSchemaResponse, ReflectScriptVisibility,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypeKind, ReflectTypePath,
    ReflectTypeRegistration, ReflectTypeRole, ReflectWriteRequest, ReflectWriteResponse,
    ReflectedValue, MAX_REFLECT_TYPE_PATH_BYTES, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
};

/// 为 wire-contract 用例组装含数值与文档字段的组件注册样本。
fn sample_registration() -> ReflectTypeRegistration {
    let fields = vec![
        ReflectFieldInfo::from_stable_keys(
            "zircon.Sample",
            "alpha",
            "alpha",
            "f32",
            ReflectEditorHint::Scalar,
        )
        .with_numeric_range(
            ReflectNumericRange::new(Some(0.0), Some(1.0), Some(0.1), Some(2)).unwrap(),
        ),
        ReflectFieldInfo::from_stable_keys(
            "zircon.Sample",
            "name",
            "name",
            "alloc::string::String",
            ReflectEditorHint::String,
        )
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

/// 通过 JSON 文本往返检查指定 DTO 的序列化和反序列化兼容性。
fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
/// 检查注册角色、插件归属、字段顺序和 canonical JSON 字段。
fn type_registration_serializes_canonical_role_and_owner() {
    let mut registration = sample_registration()
        .with_plugin_id("sample.plugin")
        .unwrap();
    registration.type_path = registration.type_path.with_module_path("zircon").unwrap();
    let serialized = serde_json::to_value(&registration).unwrap();
    let fields = serialized
        .pointer("/type_info/fields")
        .and_then(serde_json::Value::as_array)
        .unwrap();

    assert_eq!(fields[0]["name"], "alpha");
    assert_eq!(fields[1]["name"], "name");
    assert_eq!(serialized["role"], "component");
    assert!(serialized.get("is_component").is_none());
    assert!(serialized.get("is_resource").is_none());
    assert!(serialized.get("plugin_owned").is_none());
    assert_eq!(serialized["serializable"], true);
    assert_eq!(serialized["editor_visible"], true);
    assert_eq!(serialized["remote_visible"], true);
    assert!(serialized.get("plugin_id").is_none());
    assert_eq!(serialized["type_path"]["plugin_id"], "sample.plugin");
    assert_eq!(registration.role, ReflectTypeRole::Component);
    assert!(registration.is_plugin_owned());
    assert_eq!(registration.type_path.plugin_id(), Some("sample.plugin"));
    for (field, value) in [
        ("plugin_id", serde_json::json!("sample.plugin")),
        ("plugin_owned", serde_json::json!(true)),
        ("is_component", serde_json::json!(true)),
        ("is_resource", serde_json::json!(false)),
    ] {
        let mut retired_wire = serialized.clone();
        retired_wire
            .as_object_mut()
            .unwrap()
            .insert(field.to_string(), value);
        assert!(serde_json::from_value::<ReflectTypeRegistration>(retired_wire).is_err());
    }
    assert_eq!(
        round_trip::<ReflectTypeRegistration>(&registration),
        registration
    );
}

#[test]
/// 检查脚本可见性与远端可见性分别序列化，不互相推导。
fn script_visibility_is_independent_from_remote_visibility() {
    let registration = sample_registration()
        .with_remote_visible(false)
        .with_script_visibility(ReflectScriptVisibility::Public)
        .with_documentation("Shared script-visible type metadata");
    let serialized = serde_json::to_value(&registration).unwrap();

    assert!(!registration.remote_visible);
    assert_eq!(
        registration.script_visibility,
        ReflectScriptVisibility::Public
    );
    assert_eq!(serialized["remote_visible"], false);
    assert_eq!(serialized["script_visibility"], "public");
    assert_eq!(
        serialized["documentation"],
        "Shared script-visible type metadata"
    );
    assert_eq!(
        round_trip::<ReflectTypeRegistration>(&registration),
        registration
    );
}

#[test]
/// 对所有受支持值形状逐项往返并要求序列化形式带 kind 标签。
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
/// 固定值、地址、读写 DTO 与错误变体的预期 JSON 外形。
fn reflect_contract_dtos_use_expected_json_shapes() {
    let component = ReflectObjectAddress::component(42, "zircon::Name").unwrap();
    let resource = ReflectObjectAddress::resource("zircon::FrameCounter").unwrap();
    let translation_id = ReflectFieldId::from_stable_keys("zircon.Transform", "translation");
    let field = ReflectFieldValue::new(
        translation_id,
        "translation",
        ReflectedValue::Vec3([1.0, 2.0, 3.0]),
    );
    let updated_field = ReflectFieldValue::new(
        translation_id,
        "translation",
        ReflectedValue::Vec3([4.0, 5.0, 6.0]),
    );

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
                "field_id": translation_id.to_string(),
                "field_name": "translation",
                "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
            }]
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectReadRequest::new(component.clone(), translation_id)).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field_id": translation_id.to_string()
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectReadResponse::new(component.clone(), field)).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field": {
                "field_id": translation_id.to_string(),
                "field_name": "translation",
                "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
            }
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectWriteRequest::new(
            component.clone(),
            translation_id,
            ReflectedValue::Vec3([4.0, 5.0, 6.0]),
        ))
        .unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field_id": translation_id.to_string(),
            "value": { "kind": "Vec3", "value": [4.0, 5.0, 6.0] }
        })
    );
    assert_eq!(
        serde_json::to_value(ReflectWriteResponse::new(component, updated_field, true)).unwrap(),
        serde_json::json!({
            "address": { "kind": "Component", "entity": 42, "type_path": "zircon::Name" },
            "field": {
                "field_id": translation_id.to_string(),
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
/// 确认字段值 wire 记录不能只提供已退役的名称而缺少稳定 ID。
fn reflected_field_value_rejects_legacy_name_only_wire() {
    assert!(
        serde_json::from_value::<ReflectFieldValue>(serde_json::json!({
            "field_name": "translation",
            "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
        }))
        .is_err()
    );
}

#[test]
/// 确认读写请求拒绝旧的 name-only 字段寻址 wire。
fn stable_id_read_write_requests_reject_legacy_field_name_wire() {
    let address = serde_json::json!({
        "kind": "Component",
        "entity": 42,
        "type_path": "zircon::Name"
    });
    let read = serde_json::json!({
        "address": address,
        "field_name": "translation"
    });
    let write = serde_json::json!({
        "address": {
            "kind": "Component",
            "entity": 42,
            "type_path": "zircon::Name"
        },
        "field_name": "translation",
        "value": { "kind": "Vec3", "value": [1.0, 2.0, 3.0] }
    });

    assert!(serde_json::from_value::<ReflectReadRequest>(read).is_err());
    assert!(serde_json::from_value::<ReflectWriteRequest>(write).is_err());
}

#[test]
/// 检查字段元数据选项的 round-trip 与默认可省略字段的 wire 形状。
fn field_metadata_preserves_editability_defaults_and_docs() {
    let field = ReflectFieldInfo::from_stable_keys(
        "zircon.RigidBody",
        "body-type",
        "body_type",
        "RigidBodyType",
        ReflectEditorHint::Enum,
    )
    .with_aliases(vec!["body_kind".to_string()])
    .with_display_name("Body Type")
    .with_default_value(ReflectedValue::String("Dynamic".to_string()))
    .with_numeric_range(
        ReflectNumericRange::new(Some(0.0), Some(10.0), Some(0.5), Some(1)).unwrap(),
    )
    .with_enum_options(vec![
        ReflectEnumOption::new("Dynamic", "Dynamic Body"),
        ReflectEnumOption::new("Static", "Static Body").with_documentation("Does not move"),
    ])
    .with_editable(false)
    .with_serializable(true)
    .with_editor_visible(true)
    .with_documentation("Physics body type");
    let json = serde_json::to_value(ReflectFieldInfo::from_stable_keys(
        "zircon.Visibility",
        "visible",
        "visible",
        "bool",
        ReflectEditorHint::Bool,
    ))
    .unwrap();
    let round_trip = round_trip::<ReflectFieldInfo>(&field);

    assert_eq!(round_trip.display_name, "Body Type");
    assert_eq!(round_trip.id, field.id);
    assert_eq!(round_trip.aliases, vec!["body_kind".to_string()]);
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
    assert_eq!(round_trip.numeric_range.unwrap().step(), Some(0.5));
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
/// 检查稳定字段 ID 对相同键确定、不同键区分，且 wire 解析拒绝 nil UUID。
fn reflect_field_ids_are_deterministic_canonical_and_non_nil() {
    let id = ReflectFieldId::from_stable_keys("zircon.Transform", "translation");
    assert_eq!(
        id,
        ReflectFieldId::from_stable_keys("zircon.Transform", "translation")
    );
    assert_ne!(
        id,
        ReflectFieldId::from_stable_keys("zircon.Transform", "rotation")
    );
    assert_eq!(id.to_string().parse::<ReflectFieldId>().unwrap(), id);
    assert_eq!(round_trip(&id), id);
    assert_eq!(
        "00000000-0000-0000-0000-000000000000"
            .parse::<ReflectFieldId>()
            .unwrap_err(),
        ReflectFieldIdParseError::Nil
    );
}

#[test]
/// 检查数值范围构造器和反序列化共同拒绝倒置边界及无效数值。
fn numeric_range_rejects_invalid_constructor_and_wire_states() {
    assert_eq!(
        ReflectNumericRange::new(Some(f32::NAN), None, None, None),
        Err(ReflectNumericRangeError::NonFiniteMin)
    );
    assert_eq!(
        ReflectNumericRange::new(None, Some(f32::INFINITY), None, None),
        Err(ReflectNumericRangeError::NonFiniteMax)
    );
    assert_eq!(
        ReflectNumericRange::new(None, None, Some(f32::NEG_INFINITY), None),
        Err(ReflectNumericRangeError::NonFiniteStep)
    );
    assert_eq!(
        ReflectNumericRange::new(Some(2.0), Some(1.0), None, None),
        Err(ReflectNumericRangeError::Inverted { min: 2.0, max: 1.0 })
    );
    assert_eq!(
        ReflectNumericRange::new(None, None, Some(0.0), None),
        Err(ReflectNumericRangeError::NonPositiveStep { step: 0.0 })
    );
    assert_eq!(
        ReflectNumericRange::new(None, None, Some(-0.5), None),
        Err(ReflectNumericRangeError::NonPositiveStep { step: -0.5 })
    );
    assert!(
        serde_json::from_value::<ReflectNumericRange>(serde_json::json!({
            "min": 4.0,
            "max": 3.0,
            "step": 0.1,
            "precision": 2
        }))
        .is_err()
    );
}

#[test]
/// 检查对象地址、读写消息和目录请求/响应的 JSON round-trip。
fn reflect_object_address_schema_and_read_write_dtos_roundtrip() {
    let component = ReflectObjectAddress::component(42, "zircon::Name").unwrap();
    let resource = ReflectObjectAddress::resource("zircon::FrameCounter").unwrap();
    let translation_id = ReflectFieldId::from_stable_keys("zircon.Transform", "translation");
    let field = ReflectFieldValue::new(
        translation_id,
        "translation",
        ReflectedValue::Vec3([1.0, 2.0, 3.0]),
    );
    let fields_request = ReflectFieldsRequest::new(component.clone());
    let fields_response = ReflectFieldsResponse::new(component.clone(), vec![field.clone()]);
    let read_request = ReflectReadRequest::new(component.clone(), translation_id);
    let read_response = ReflectReadResponse::new(component.clone(), field.clone());
    let write_request = ReflectWriteRequest::new(
        component.clone(),
        translation_id,
        ReflectedValue::Vec3([4.0, 5.0, 6.0]),
    );
    let write_response = ReflectWriteResponse::new(
        component.clone(),
        ReflectFieldValue::new(
            translation_id,
            "translation",
            ReflectedValue::Vec3([4.0, 5.0, 6.0]),
        ),
        true,
    );
    let schema_filter = ReflectSchemaFilter::remote_visible();
    let schema_request = ReflectSchemaRequest::new(schema_filter.clone());
    let schema_catalog =
        ReflectSchemaCatalog::try_new(vec![ReflectSchemaCatalogEntry::new(sample_registration())])
            .unwrap();
    let schema_response =
        ReflectSchemaResponse::new(schema_catalog.fingerprint(), vec![sample_registration()]);

    assert!(matches!(component, ReflectObjectAddress::Component { .. }));
    assert_eq!(component.type_path(), "zircon::Name");
    assert_eq!(resource.type_path(), "zircon::FrameCounter");
    assert!(schema_filter.remote_visible);
    assert!(schema_filter.include_components);
    assert_eq!(
        schema_response.catalog_algorithm_version,
        REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION
    );
    assert_eq!(
        schema_response.catalog_fingerprint,
        schema_catalog.fingerprint()
    );
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
/// 检查错误文本保留类型、字段、实体及地址种类等定位上下文。
fn reflect_error_display_includes_type_field_and_entity_context() {
    let errors = [
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
/// 固定类型路径与类型类别的规范 wire 表示，并核对类型信息便捷构造器。
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

#[test]
/// 检查 Rust/VM 路径共用构造与反序列化校验语法并拒绝非法路径。
fn reflect_type_path_uses_one_validated_constructor_and_wire_grammar() {
    let rust_path = ReflectTypePath::new("zircon::scene::Name", "Name")
        .unwrap()
        .with_module_path("zircon::scene")
        .unwrap()
        .with_plugin_id("sample.plugin")
        .unwrap();
    let vm_path = ReflectTypePath::new("gameplay.Component.Health", "Health").unwrap();
    let third_party_vm_path =
        ReflectTypePath::new("third-party.Component.Health", "Health").unwrap();

    assert_eq!(rust_path.type_path(), "zircon::scene::Name");
    assert_eq!(rust_path.short_type_path(), "Name");
    assert_eq!(rust_path.module_path(), Some("zircon::scene"));
    assert_eq!(rust_path.plugin_id(), Some("sample.plugin"));
    assert_eq!(round_trip::<ReflectTypePath>(&rust_path), rust_path);
    assert_eq!(round_trip::<ReflectTypePath>(&vm_path), vm_path);
    assert_eq!(
        round_trip::<ReflectTypePath>(&third_party_vm_path),
        third_party_vm_path
    );
    assert!(matches!(
        ReflectTypePath::new("zircon::scene.Name", "Name"),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(matches!(
        ReflectTypePath::new("zircon::scene::Name", "SceneName"),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(matches!(
        ReflectTypePath::new("zircon::scene::Vec<Name>", "Vec<Name>"),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(matches!(
        ReflectTypePath::new("third-party.Component.Health-Value", "Health-Value"),
        Err(ReflectError::InvalidTypePath { .. })
    ));
    assert!(ReflectTypePath::new("a".repeat(MAX_REFLECT_TYPE_PATH_BYTES + 1), "a").is_err());
    assert!(ReflectTypePath::new("zircon::Name", "Name")
        .unwrap()
        .with_module_path("zircon::other")
        .is_err());
    assert!(ReflectTypePath::new("zircon::Name", "Name")
        .unwrap()
        .with_plugin_id(" sample.plugin")
        .is_err());
    assert!(ReflectTypePath::new("zircon::Name", "Name")
        .unwrap()
        .with_plugin_id("Sample.Plugin")
        .is_err());

    for invalid in [
        serde_json::json!({
            "type_path": " ",
            "short_type_path": "Name"
        }),
        serde_json::json!({
            "type_path": "zircon::scene.Name",
            "short_type_path": "Name"
        }),
        serde_json::json!({
            "type_path": "zircon::scene::Name",
            "short_type_path": "SceneName"
        }),
        serde_json::json!({
            "type_path": "zircon::scene::Name",
            "short_type_path": "Name",
            "module_path": "zircon::other"
        }),
        serde_json::json!({
            "type_path": "zircon::scene::Name",
            "short_type_path": "Name",
            "plugin_id": "sample plugin"
        }),
        serde_json::json!({
            "type_path": "zircon::scene::Name",
            "short_type_path": "Name",
            "unexpected": true
        }),
    ] {
        assert!(serde_json::from_value::<ReflectTypePath>(invalid).is_err());
    }
}
