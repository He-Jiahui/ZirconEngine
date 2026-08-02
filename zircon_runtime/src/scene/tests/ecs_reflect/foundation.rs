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
    EntityId, NodeKind, ReflectComponent, ReflectResource, RuntimeTypeRegistration, TypeRegistry,
    World, json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected,
};

mod address_routing;
mod fixed_lights_name;
mod fixed_registry;
mod fixed_render_physics;
mod fixed_transform_active;
mod registry;
mod value_conversion;
mod versioned_json;

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
        estimate_stage_clone_bytes: None,
        stage_clone: None,
        ensure: None,
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
