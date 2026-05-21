use std::collections::BTreeMap;

use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectObjectAddress,
    ReflectReadRequest, ReflectSchemaFilter, ReflectSchemaRequest, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::{Transform, Vec2, Vec3};
use crate::scene::components::{
    ActiveSelf, AmbientLight, LocalTransform, Name, RectLight, RenderLayerMask, RigidBodyComponent,
    RigidBodyType,
};
use crate::scene::{
    json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected, EntityId, NodeKind, ReflectComponent, ReflectResource,
    RuntimeTypeRegistration, TypeRegistry, World,
};

#[test]
fn empty_world_builds_runtime_only_type_registry() {
    let world = World::empty();

    assert!(!world.type_registry().is_empty());
}

#[test]
fn type_registry_rejects_duplicate_full_type_paths() {
    let mut registry = TypeRegistry::default();
    let registration = metadata_registration("plugin_a::Short", "Short");

    registry
        .register(RuntimeTypeRegistration::metadata(registration.clone()))
        .expect("first registration should be accepted");
    assert!(registry.contains_type_path("plugin_a::Short"));
    let error = registry
        .register(RuntimeTypeRegistration::metadata(registration))
        .expect_err("duplicate full type path should be rejected");

    assert_eq!(
        error,
        ReflectError::DuplicateTypePath {
            type_path: "plugin_a::Short".to_string(),
        }
    );
}

#[test]
fn type_registry_short_path_lookup_reports_ambiguity() {
    let mut registry = TypeRegistry::default();

    registry
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_a::Short",
            "Short",
        )))
        .expect("first short path should be accepted");
    assert!(registry.contains("Short"));
    assert!(!registry.contains_type_path("Short"));
    registry
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_b::Short",
            "Short",
        )))
        .expect("second full type path should be accepted");

    let caller_owned_type_path = String::from("plugin_a::Short");
    let resolved = registry.resolve(caller_owned_type_path.as_str()).unwrap();
    drop(caller_owned_type_path);

    assert_eq!(resolved, "plugin_a::Short");
    assert_eq!(
        registry.resolve("plugin_b::Short").unwrap(),
        "plugin_b::Short"
    );
    assert!(registry.contains("plugin_a::Short"));
    assert!(!registry.contains("Short"));
    assert!(!registry.contains_type_path("Short"));
    assert_eq!(
        registry
            .iter()
            .map(|registration| registration.registration.type_path.type_path.as_str())
            .collect::<Vec<_>>(),
        vec!["plugin_a::Short", "plugin_b::Short"]
    );
    assert_eq!(
        registry
            .resolve("Short")
            .expect_err("short path is ambiguous"),
        ReflectError::AmbiguousShortTypePath {
            short_type_path: "Short".to_string(),
        }
    );

    registry.clear();
    assert!(registry.is_empty());
    assert_eq!(
        registry.resolve("Short").expect_err("registry was cleared"),
        ReflectError::UnknownType {
            type_path: "Short".to_string(),
        }
    );
}

#[test]
fn runtime_type_registration_compares_adapter_presence_not_identity() {
    let metadata = metadata_registration("plugin_a::RuntimeOnly", "RuntimeOnly");
    let metadata_only = RuntimeTypeRegistration::metadata(metadata.clone());
    let with_component = RuntimeTypeRegistration {
        registration: metadata.clone(),
        component: Some(dummy_component_adapter()),
        resource: None,
    };
    let with_resource = RuntimeTypeRegistration {
        registration: metadata,
        component: Some(dummy_component_adapter()),
        resource: Some(dummy_resource_adapter()),
    };

    assert_ne!(metadata_only, with_component);
    assert_ne!(with_component, with_resource);
    assert!(format!("{with_resource:?}").contains("has_component_adapter: true"));
    assert!(format!("{with_resource:?}").contains("has_resource_adapter: true"));
}

#[test]
fn world_serialization_skips_reflection_registry_and_rebuilds_it_on_load() {
    let mut world = World::empty();
    let builtin_count = world.type_registry().iter().count();
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_a::RuntimeOnly",
            "RuntimeOnly",
        )))
        .expect("test registration should be accepted");

    let json = serde_json::to_string(&world).expect("world should serialize");

    assert!(!json.contains("type_registry"));
    assert!(!json.contains("registrations"));
    assert!(!json.contains("RuntimeOnly"));

    let loaded: World = serde_json::from_str(&json).expect("world should deserialize");

    assert_eq!(loaded.type_registry().iter().count(), builtin_count);
    assert!(loaded
        .type_registry()
        .contains_type_path("zircon_runtime::scene::components::Name"));
}

#[test]
fn scene_property_values_convert_to_reflected_values() {
    let cases = [
        (ScenePropertyValue::Bool(true), ReflectedValue::Bool(true)),
        (ScenePropertyValue::Integer(-7), ReflectedValue::Integer(-7)),
        (ScenePropertyValue::Unsigned(9), ReflectedValue::Unsigned(9)),
        (ScenePropertyValue::Scalar(1.5), ReflectedValue::Scalar(1.5)),
        (
            ScenePropertyValue::String("name".to_string()),
            ReflectedValue::String("name".to_string()),
        ),
        (
            ScenePropertyValue::Enum("Dynamic".to_string()),
            ReflectedValue::Enum("Dynamic".to_string()),
        ),
        (
            ScenePropertyValue::Vec2([1.0, 2.0]),
            ReflectedValue::Vec2([1.0, 2.0]),
        ),
        (
            ScenePropertyValue::Vec3([1.0, 2.0, 3.0]),
            ReflectedValue::Vec3([1.0, 2.0, 3.0]),
        ),
        (
            ScenePropertyValue::Vec4([1.0, 2.0, 3.0, 4.0]),
            ReflectedValue::Vec4([1.0, 2.0, 3.0, 4.0]),
        ),
        (
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
            ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ),
        (
            ScenePropertyValue::Entity(Some(42)),
            ReflectedValue::Entity(Some(42)),
        ),
        (
            ScenePropertyValue::Entity(None),
            ReflectedValue::Entity(None),
        ),
        (
            ScenePropertyValue::Resource("mesh://cube".to_string()),
            ReflectedValue::Resource("mesh://cube".to_string()),
        ),
    ];

    for (scene_value, reflected_value) in cases {
        assert_eq!(
            reflected_from_scene_value(scene_value).expect("scene value should convert"),
            reflected_value
        );
    }
}

#[test]
fn reflected_values_convert_to_scene_property_values_when_supported() {
    let cases = [
        (ReflectedValue::Bool(true), ScenePropertyValue::Bool(true)),
        (ReflectedValue::Integer(-7), ScenePropertyValue::Integer(-7)),
        (ReflectedValue::Unsigned(9), ScenePropertyValue::Unsigned(9)),
        (ReflectedValue::Scalar(1.5), ScenePropertyValue::Scalar(1.5)),
        (
            ReflectedValue::String("name".to_string()),
            ScenePropertyValue::String("name".to_string()),
        ),
        (
            ReflectedValue::Enum("Dynamic".to_string()),
            ScenePropertyValue::Enum("Dynamic".to_string()),
        ),
        (
            ReflectedValue::Vec2([1.0, 2.0]),
            ScenePropertyValue::Vec2([1.0, 2.0]),
        ),
        (
            ReflectedValue::Vec3([1.0, 2.0, 3.0]),
            ScenePropertyValue::Vec3([1.0, 2.0, 3.0]),
        ),
        (
            ReflectedValue::Vec4([1.0, 2.0, 3.0, 4.0]),
            ScenePropertyValue::Vec4([1.0, 2.0, 3.0, 4.0]),
        ),
        (
            ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ),
        (
            ReflectedValue::Entity(Some(42)),
            ScenePropertyValue::Entity(Some(42)),
        ),
        (
            ReflectedValue::Entity(None),
            ScenePropertyValue::Entity(None),
        ),
        (
            ReflectedValue::Resource("mesh://cube".to_string()),
            ScenePropertyValue::Resource("mesh://cube".to_string()),
        ),
    ];

    for (reflected_value, scene_value) in cases {
        assert_eq!(
            scene_value_from_reflected(reflected_value).expect("reflected value should convert"),
            scene_value
        );
    }

    assert_eq!(
        scene_value_from_reflected(ReflectedValue::Null).expect_err("null is not a scene value"),
        ReflectError::UnsupportedConversion {
            source: "ReflectedValue::Null".to_string(),
            target: "ScenePropertyValue".to_string(),
        }
    );
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::List(Vec::new())),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Map(BTreeMap::new())),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Json(json!({ "arbitrary": true }))),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Scalar(f32::NAN)),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Quaternion([0.0, 0.0, f32::INFINITY, 1.0])),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
}

#[test]
fn reflected_json_conversion_rejects_non_finite_scalars() {
    assert_eq!(
        reflected_from_json(json!({ "nested": [1, true] })),
        ReflectedValue::Json(json!({ "nested": [1, true] }))
    );
    assert_eq!(
        json_from_reflected(ReflectedValue::Entity(Some(7))).expect("entity should serialize"),
        json!({ "kind": "Entity", "value": 7 })
    );
    assert_eq!(
        json_from_reflected(ReflectedValue::Vec3([1.0, 2.0, 3.0]))
            .expect("vector should serialize"),
        json!({ "kind": "Vec3", "value": [1.0, 2.0, 3.0] })
    );
    assert_eq!(
        json_from_reflected(ReflectedValue::Null).expect("null should serialize"),
        json!({ "kind": "Null" })
    );
    assert_eq!(
        json_from_reflected(ReflectedValue::Json(json!({ "nested": [1, true] })))
            .expect("arbitrary JSON should serialize as tagged DTO"),
        json!({ "kind": "Json", "value": { "nested": [1, true] } })
    );

    assert_eq!(
        json_from_reflected(ReflectedValue::Scalar(f32::INFINITY))
            .expect_err("non-finite scalar should not serialize"),
        ReflectError::UnsupportedConversion {
            source: "ReflectedValue::Scalar".to_string(),
            target: "serde_json::Value".to_string(),
        }
    );
    assert!(matches!(
        json_from_reflected(ReflectedValue::List(vec![ReflectedValue::Vec2([
            1.0,
            f32::NAN,
        ])])),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
}

#[test]
fn animation_parameter_conversion_returns_structured_error() {
    assert_eq!(
        reflected_from_scene_value(ScenePropertyValue::AnimationParameter(
            AnimationParameterValue::Bool(true),
        ))
        .expect_err("animation parameters are outside M8.3 value conversion"),
        ReflectError::UnsupportedConversion {
            source: "ScenePropertyValue::AnimationParameter".to_string(),
            target: "ReflectedValue".to_string(),
        }
    );
}

#[test]
fn world_reflection_routes_component_and_resource_addresses() {
    let mut world = World::empty();
    world.type_registry_mut_for_tests().clear();
    let entity = world.spawn_node(NodeKind::Mesh);

    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration: typed_registration("plugin_a::ProbeComponent", "ProbeComponent")
                .as_component(),
            component: Some(dummy_component_adapter()),
            resource: None,
        })
        .expect("component registration should be accepted");
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration: typed_registration("plugin_a::ProbeResource", "ProbeResource")
                .as_resource()
                .with_remote_visible(true),
            component: None,
            resource: Some(dummy_resource_adapter()),
        })
        .expect("resource registration should be accepted");
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(
            typed_registration("plugin_a::PluginHidden", "PluginHidden")
                .as_component()
                .with_plugin_owned(true),
        ))
        .expect("plugin metadata should be accepted");

    let listed = world
        .list_reflect_types(ReflectSchemaRequest::new(ReflectSchemaFilter {
            include_components: true,
            include_resources: true,
            include_plugin_owned: false,
            ..ReflectSchemaFilter::default()
        }))
        .expect("schema list should route through registry")
        .registrations
        .into_iter()
        .map(|registration| registration.type_path.type_path)
        .collect::<Vec<_>>();
    assert_eq!(
        listed,
        vec!["plugin_a::ProbeComponent", "plugin_a::ProbeResource"]
    );
    assert_eq!(
        world
            .reflect_schema("ProbeResource")
            .expect("short type path should resolve")
            .type_path
            .type_path,
        "plugin_a::ProbeResource"
    );

    let component_address = ReflectObjectAddress::component(entity, "ProbeComponent")
        .expect("component address should be valid");
    let component_fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(component_address.clone()),
        )
        .expect("component fields should route to component adapter");
    assert_eq!(
        component_fields.fields,
        vec![ReflectFieldValue::new(
            "entity",
            ReflectedValue::Unsigned(entity)
        )]
    );
    let component_write = world
        .reflect_write(ReflectWriteRequest::new(
            component_address.clone(),
            "entity",
            ReflectedValue::Unsigned(entity),
        ))
        .expect("component write should route to component adapter and read back");
    assert!(!component_write.changed);
    assert_eq!(
        component_write.field,
        ReflectFieldValue::new("entity", ReflectedValue::Unsigned(entity))
    );

    let resource_address = ReflectObjectAddress::resource("plugin_a::ProbeResource")
        .expect("resource address should be valid");
    let resource_read = world
        .reflect_read(ReflectReadRequest::new(resource_address.clone(), "enabled"))
        .expect("resource read should route to resource adapter");
    assert_eq!(
        resource_read.field,
        ReflectFieldValue::new("enabled", ReflectedValue::Bool(true))
    );
    let resource_write = world
        .reflect_write(ReflectWriteRequest::new(
            resource_address,
            "enabled",
            ReflectedValue::Bool(true),
        ))
        .expect("resource write should route to resource adapter and read back");
    assert!(!resource_write.changed);
    assert_eq!(
        resource_write.field,
        ReflectFieldValue::new("enabled", ReflectedValue::Bool(true))
    );

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(component_address, "missing"))
            .expect_err("unknown field should propagate from adapter"),
        ReflectError::UnknownField {
            type_path: "plugin_a::ProbeComponent".to_string(),
            field_name: "missing".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_fields(
                zircon_runtime_interface::reflect::ReflectFieldsRequest::new(
                    ReflectObjectAddress::component(entity, "plugin_a::ProbeResource")
                        .expect("address should be valid"),
                )
            )
            .expect_err("resource registration cannot be addressed as component"),
        ReflectError::AddressKindMismatch {
            expected: "component `plugin_a::ProbeResource`".to_string(),
            actual: "non-component `plugin_a::ProbeResource`".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_fields(
                zircon_runtime_interface::reflect::ReflectFieldsRequest::new(
                    ReflectObjectAddress::component(entity, "plugin_a::PluginHidden")
                        .expect("address should be valid"),
                )
            )
            .expect_err("metadata-only component has no adapter"),
        ReflectError::NoComponentAdapter {
            type_path: "plugin_a::PluginHidden".to_string(),
        }
    );
}

#[test]
fn fixed_component_registrations_exist_in_empty_world() {
    let world = World::empty();
    let expected = [
        "zircon_runtime::scene::components::ActiveSelf",
        "zircon_runtime::scene::components::AmbientLight",
        "zircon_runtime::scene::components::LocalTransform",
        "zircon_runtime::scene::components::Name",
        "zircon_runtime::scene::components::RectLight",
        "zircon_runtime::scene::components::RenderLayerMask",
        "zircon_runtime::scene::components::RigidBodyComponent",
    ];

    for type_path in expected {
        let registration = world
            .reflect_schema(type_path)
            .expect("fixed component schema should be registered");
        assert!(registration.is_component);
        assert!(world
            .type_registry()
            .runtime_registration(type_path)
            .expect("fixed runtime registration should exist")
            .component
            .is_some());
    }
}

#[test]
fn ambient_and_rect_light_reflection_roundtrips_authoring_fields() {
    let mut world = World::empty();
    let ambient = world.spawn_node(NodeKind::AmbientLight);
    let rect = world.spawn_node(NodeKind::RectLight);
    let ambient_address = fixed_component_address(ambient, "AmbientLight");
    let rect_address = fixed_component_address(rect, "RectLight");

    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            "color",
            ReflectedValue::Vec3([0.05, 0.06, 0.07]),
        ))
        .expect("ambient color should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            "intensity",
            ReflectedValue::Scalar(0.35),
        ))
        .expect("ambient intensity should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            "affects_lightmapped_meshes",
            ReflectedValue::Bool(false),
        ))
        .expect("ambient lightmap flag should be writable");

    let ambient_light = world.get::<AmbientLight>(ambient).unwrap();
    assert_eq!(ambient_light.color, Vec3::new(0.05, 0.06, 0.07));
    assert_eq!(ambient_light.intensity, 0.35);
    assert!(!ambient_light.affects_lightmapped_meshes);
    assert!(world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(ambient_address)
        )
        .expect("ambient fields should be enumerable")
        .fields
        .contains(&ReflectFieldValue::new(
            "affects_lightmapped_meshes",
            ReflectedValue::Bool(false)
        )));

    world
        .reflect_write(ReflectWriteRequest::new(
            rect_address.clone(),
            "range",
            ReflectedValue::Scalar(16.0),
        ))
        .expect("rect range should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            rect_address.clone(),
            "size",
            ReflectedValue::Vec2([2.0, 0.5]),
        ))
        .expect("rect size should be writable");

    let rect_light = world.get::<RectLight>(rect).unwrap();
    assert_eq!(rect_light.range, 16.0);
    assert_eq!(rect_light.size, Vec2::new(2.0, 0.5));
    assert!(world
        .reflect_fields(zircon_runtime_interface::reflect::ReflectFieldsRequest::new(rect_address))
        .expect("rect fields should be enumerable")
        .fields
        .contains(&ReflectFieldValue::new(
            "size",
            ReflectedValue::Vec2([2.0, 0.5])
        )));
}

#[test]
fn name_component_reads_and_writes_through_world_reflection() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    let address =
        ReflectObjectAddress::component(entity, "zircon_runtime::scene::components::Name")
            .expect("fixed component full-path address");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(address.clone(), "value"))
            .expect("name should be readable")
            .field,
        ReflectFieldValue::new("value", ReflectedValue::String("Mesh 1".to_string()))
    );

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address,
            "value",
            ReflectedValue::String("Reflected Name".to_string()),
        ))
        .expect("name should be writable");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new(
            "value",
            ReflectedValue::String("Reflected Name".to_string())
        )
    );
    assert_eq!(world.get::<Name>(entity).unwrap().0, "Reflected Name");
}

#[test]
fn active_self_reflection_write_marks_active_dirty_state() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    assert_eq!(world.active_in_hierarchy(entity), Some(true));

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "ActiveSelf"),
            "value",
            ReflectedValue::Bool(false),
        ))
        .expect("active state should be writable");

    assert!(response.changed);
    assert_eq!(world.get::<ActiveSelf>(entity), Some(&ActiveSelf(false)));
    assert!(world.has_pending_scene_systems());
    assert_eq!(world.active_in_hierarchy(entity), Some(false));
}

#[test]
fn local_transform_reflection_write_marks_transform_dirty_state() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "translation",
            ReflectedValue::Vec3([5.0, 6.0, 7.0]),
        ))
        .expect("local transform translation should be writable");

    assert!(response.changed);
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .unwrap()
            .transform
            .translation,
        Vec3::new(5.0, 6.0, 7.0)
    );
    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(entity).unwrap().translation,
        Vec3::new(5.0, 6.0, 7.0)
    );

    let scale_response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "scale",
            ReflectedValue::Vec3([2.0, 3.0, 4.0]),
        ))
        .expect("local transform scale should be writable");

    assert!(scale_response.changed);
    assert_eq!(
        world.get::<LocalTransform>(entity).unwrap().transform.scale,
        Vec3::new(2.0, 3.0, 4.0)
    );
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    let no_op_scale = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "scale",
            ReflectedValue::Vec3([2.0, 3.0, 4.0]),
        ))
        .expect("same local transform scale should be accepted as unchanged");
    assert!(!no_op_scale.changed);
    assert!(!world.has_pending_scene_systems());
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "translation",
            ReflectedValue::Vec3([f32::NAN, 0.0, 0.0]),
        )),
        Err(ReflectError::TypeMismatch { expected, .. }) if expected == "finite Vec3"
    ));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn local_transform_rotation_is_readable_but_not_writable_in_m8() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .insert(
            entity,
            LocalTransform {
                transform: Transform::default(),
            },
        )
        .unwrap();
    let address = fixed_component_address(entity, "LocalTransform");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(address.clone(), "rotation"))
            .expect("rotation should be readable")
            .field,
        ReflectFieldValue::new("rotation", ReflectedValue::Vec4([0.0, 0.0, 0.0, 1.0]))
    );
    assert!(
        !world
            .reflect_schema("LocalTransform")
            .expect("schema should resolve by short type path")
            .type_info
            .fields
            .iter()
            .find(|field| field.name == "rotation")
            .expect("rotation schema should exist")
            .editable
    );
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                address,
                "rotation",
                ReflectedValue::Vec4([0.0, 0.0, 0.0, 1.0]),
            ))
            .expect_err("rotation writes are deferred until a later milestone"),
        ReflectError::NonEditableField {
            type_path: "zircon_runtime::scene::components::LocalTransform".to_string(),
            field_name: "rotation".to_string(),
        }
    );
}

#[test]
fn render_layer_mask_reflection_roundtrips_unsigned_mask() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    let address = fixed_component_address(entity, "RenderLayerMask");

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "mask",
            ReflectedValue::Unsigned(0x0000_00ff),
        ))
        .expect("render layer mask should be writable");

    assert!(response.changed);
    assert_eq!(
        world.get::<RenderLayerMask>(entity),
        Some(&RenderLayerMask(0xff))
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(address.clone(), "mask"))
            .expect("mask should read back")
            .field,
        ReflectFieldValue::new("mask", ReflectedValue::Unsigned(0xff))
    );
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            address,
            "mask",
            ReflectedValue::Unsigned(u32::MAX as u64 + 1),
        )),
        Err(ReflectError::TypeMismatch { .. })
    ));
}

#[test]
fn rigid_body_reflection_exposes_selected_safe_fields() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .insert(
            entity,
            RigidBodyComponent {
                body_type: RigidBodyType::Kinematic,
                linear_velocity: Vec3::new(1.0, 2.0, 3.0),
                lock_rotation: [true, false, true],
                ..RigidBodyComponent::default()
            },
        )
        .unwrap();
    let address = fixed_component_address(entity, "RigidBodyComponent");

    let fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(address.clone()),
        )
        .expect("rigid body fields should be enumerable")
        .fields;

    assert!(fields.contains(&ReflectFieldValue::new(
        "body_type",
        ReflectedValue::Enum("Kinematic".to_string())
    )));
    assert!(fields.contains(&ReflectFieldValue::new(
        "linear_velocity",
        ReflectedValue::Vec3([1.0, 2.0, 3.0])
    )));
    assert!(fields.contains(&ReflectFieldValue::new(
        "lock_rotation",
        ReflectedValue::List(vec![
            ReflectedValue::Bool(true),
            ReflectedValue::Bool(false),
            ReflectedValue::Bool(true),
        ])
    )));
    let rigid_schema = world.reflect_schema("RigidBodyComponent").unwrap();
    assert!(
        !rigid_schema
            .type_info
            .fields
            .iter()
            .find(|field| field.name == "body_type")
            .unwrap()
            .editable
    );
    let lock_rotation_schema = rigid_schema
        .type_info
        .fields
        .iter()
        .find(|field| field.name == "lock_rotation")
        .unwrap();
    assert!(!lock_rotation_schema.editable);
    assert_eq!(lock_rotation_schema.value_type_path, "List<Bool>");
    assert!(matches!(
        lock_rotation_schema.editor_hint,
        ReflectEditorHint::None
    ));
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "mass",
            ReflectedValue::Scalar(9.5),
        ))
        .expect("mass should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "linear_damping",
            ReflectedValue::Scalar(0.25),
        ))
        .expect("linear damping should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "angular_damping",
            ReflectedValue::Scalar(0.5),
        ))
        .expect("angular damping should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "gravity_scale",
            ReflectedValue::Scalar(0.75),
        ))
        .expect("gravity scale should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "can_sleep",
            ReflectedValue::Bool(false),
        ))
        .expect("can_sleep should be writable");

    let rigid_body = world.get::<RigidBodyComponent>(entity).unwrap();
    assert_eq!(rigid_body.mass, 9.5);
    assert_eq!(rigid_body.linear_damping, 0.25);
    assert_eq!(rigid_body.angular_damping, 0.5);
    assert_eq!(rigid_body.gravity_scale, 0.75);
    assert!(!rigid_body.can_sleep);
    let unchanged_mass = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "mass",
            ReflectedValue::Scalar(9.5),
        ))
        .expect("same rigid body mass should be accepted as unchanged");
    assert!(!unchanged_mass.changed);
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "mass",
            ReflectedValue::Scalar(f32::INFINITY),
        )),
        Err(ReflectError::TypeMismatch { expected, .. }) if expected == "finite Scalar"
    ));
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                address,
                "linear_velocity",
                ReflectedValue::Vec3([0.0, 0.0, 0.0]),
            ))
            .expect_err("linear velocity is read-only"),
        ReflectError::NonEditableField {
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
            field_name: "linear_velocity".to_string(),
        }
    );
}

#[test]
fn unknown_fixed_field_returns_structured_error() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(entity, "Name"),
                "missing",
            ))
            .expect_err("unknown fields should be structured"),
        ReflectError::UnknownField {
            type_path: "zircon_runtime::scene::components::Name".to_string(),
            field_name: "missing".to_string(),
        }
    );
}

#[test]
fn missing_fixed_component_returns_structured_error() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.remove::<RigidBodyComponent>(entity).unwrap();

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(entity, "RigidBodyComponent"),
                "mass",
            ))
            .expect_err("missing fixed components should be structured"),
        ReflectError::MissingComponent {
            entity,
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
        }
    );
    let rigid_body_adapter = world
        .type_registry()
        .runtime_registration("RigidBodyComponent")
        .expect("rigid body registration should resolve")
        .component
        .clone()
        .expect("rigid body registration should have component adapter");
    assert_eq!(
        rigid_body_adapter
            .remove(&mut world, entity)
            .expect_err("missing fixed component removals should be structured"),
        ReflectError::MissingComponent {
            entity,
            type_path: "zircon_runtime::scene::components::RigidBodyComponent".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                fixed_component_address(999_999, "Name"),
                "value",
            ))
            .expect_err("missing entities should be structured"),
        ReflectError::MissingEntity { entity: 999_999 }
    );
}

fn metadata_registration(type_path: &str, short_type_path: &str) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path).expect("valid test type path"),
        short_type_path,
        ReflectTypeInfo::opaque(),
        ReflectSerializationStrategy::Json,
    )
}

fn typed_registration(type_path: &str, short_type_path: &str) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path).expect("valid test type path"),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            "entity",
            "u64",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::Value,
    )
}

fn fixed_component_address(entity: EntityId, short_type_path: &str) -> ReflectObjectAddress {
    ReflectObjectAddress::component(entity, short_type_path).expect("fixed component address")
}

fn dummy_component_adapter() -> ReflectComponent {
    ReflectComponent::new(
        "plugin_a::ProbeComponent",
        dummy_component_contains,
        dummy_component_read_field,
        dummy_component_read_fields,
        dummy_component_write_field,
        dummy_component_remove,
    )
}

fn dummy_component_contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.contains_entity(entity)
}

fn dummy_component_read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    if !world.contains_entity(entity) {
        return Err(ReflectError::MissingEntity { entity });
    }
    match field_name {
        "entity" => Ok(ReflectedValue::Unsigned(entity)),
        _ => Err(ReflectError::UnknownField {
            type_path: "plugin_a::ProbeComponent".to_string(),
            field_name: field_name.to_string(),
        }),
    }
}

fn dummy_component_read_fields(
    world: &World,
    entity: EntityId,
    type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "entity",
        dummy_component_read_field(world, entity, type_path, "entity")?,
    )])
}

fn dummy_component_write_field(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = dummy_component_read_field(world, entity, type_path, field_name)?;
    Ok(current != value)
}

fn dummy_component_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    Ok(world.contains_entity(entity))
}

fn dummy_resource_adapter() -> ReflectResource {
    ReflectResource {
        contains: dummy_resource_contains,
        read_field: dummy_resource_read_field,
        read_fields: dummy_resource_read_fields,
        write_field: dummy_resource_write_field,
    }
}

fn dummy_resource_contains(_: &World) -> bool {
    true
}

fn dummy_resource_read_field(_: &World, field_name: &str) -> Result<ReflectedValue, ReflectError> {
    match field_name {
        "enabled" => Ok(ReflectedValue::Bool(true)),
        _ => Err(ReflectError::UnknownField {
            type_path: "plugin_a::ProbeResource".to_string(),
            field_name: field_name.to_string(),
        }),
    }
}

fn dummy_resource_read_fields(_: &World) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "enabled",
        ReflectedValue::Bool(true),
    )])
}

fn dummy_resource_write_field(
    world: &mut World,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = dummy_resource_read_field(world, field_name)?;
    Ok(current != value)
}
