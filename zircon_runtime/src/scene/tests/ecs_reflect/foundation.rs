//! 为反射目录与地址路由测试提供受控元数据和虚拟适配器；实际组件副作用由相邻 World 测试核对。

use std::collections::BTreeMap;

use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectEnumOption, ReflectError, ReflectFieldId, ReflectFieldInfo,
    ReflectFieldValue, ReflectNumericRange, ReflectObjectAddress, ReflectReadRequest,
    ReflectSchemaFilter, ReflectSchemaRequest, ReflectSerializationStrategy, ReflectTypeInfo,
    ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
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

#[path = "foundation/address_routing.rs"]
mod address_routing;
#[path = "foundation/camera.rs"]
mod camera;
#[path = "foundation/fixed_lights_name.rs"]
mod fixed_lights_name;
#[path = "foundation/fixed_registry.rs"]
mod fixed_registry;
#[path = "foundation/fixed_render_physics.rs"]
mod fixed_render_physics;
#[path = "foundation/fixed_transform_active.rs"]
mod fixed_transform_active;
#[path = "foundation/native_display.rs"]
mod native_display;
#[path = "foundation/registry.rs"]
mod registry;
#[path = "foundation/value_conversion.rs"]
mod value_conversion;
#[path = "foundation/versioned_json.rs"]
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
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            type_path,
            "entity",
            "entity",
            "u64",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::Value,
    )
}

fn typed_resource_registration(type_path: &str, short_type_path: &str) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path).expect("valid test type path"),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            type_path,
            "enabled",
            "enabled",
            "Bool",
            ReflectEditorHint::Bool,
        )]),
        ReflectSerializationStrategy::Value,
    )
}

fn reflected_field_id(owner_key: &str, field_key: &str) -> ReflectFieldId {
    ReflectFieldId::from_stable_keys(owner_key, field_key)
}

fn fixed_component_address(entity: EntityId, short_type_path: &str) -> ReflectObjectAddress {
    ReflectObjectAddress::component(entity, short_type_path).expect("fixed component address")
}

// 组件回调同时提供字段名和稠密槽位接口；写入回调只比较并报告 changed，不保存新值。
fn dummy_component_adapter() -> ReflectComponent {
    ReflectComponent::new(
        "plugin_a::ProbeComponent",
        dummy_component_contains,
        dummy_component_read_field,
        dummy_component_write_field,
        dummy_component_remove,
    )
    .with_dense_field_slots(
        dummy_component_read_field_by_slot,
        dummy_component_write_field_by_slot,
    )
}

// 此虚拟 contains 只以实体是否存活作为判断，不模拟实体上具体组件的存储。
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

fn dummy_component_read_field_by_slot(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    if !world.contains_entity(entity) {
        return Err(ReflectError::MissingEntity { entity });
    }
    match field_slot {
        0 => Ok(ReflectedValue::Unsigned(entity)),
        _ => Err(ReflectError::UnknownField {
            type_path: "plugin_a::ProbeComponent".to_string(),
            field_name: format!("#{field_slot}"),
        }),
    }
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

fn dummy_component_write_field_by_slot(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = dummy_component_read_field_by_slot(world, entity, type_path, field_slot)?;
    Ok(current != value)
}

// 测试 remove 回调仅返回实体存在性，不会从 World 中移除组件。
fn dummy_component_remove(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    Ok(world.contains_entity(entity))
}

// 资源 fixture 固定返回存在和 enabled=true；写回只比较输入，不持久化资源状态。
fn dummy_resource_adapter() -> ReflectResource {
    ReflectResource {
        estimate_stage_clone_bytes: None,
        stage_clone: None,
        transfer_preflight: dummy_resource_transfer_preflight,
        ensure: None,
        contains: dummy_resource_contains,
        read_field: dummy_resource_read_field,
        read_field_by_slot: dummy_resource_read_field_by_slot,
        write_field_by_slot: dummy_resource_write_field_by_slot,
        write_fields_by_slot: dummy_resource_write_fields_by_slot,
    }
}

fn dummy_resource_contains(_: &World) -> bool {
    true
}

fn dummy_resource_transfer_preflight(_: &mut World, _: &mut World) -> Result<(), ReflectError> {
    Ok(())
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

fn dummy_resource_read_field_by_slot(
    _: &World,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match field_slot {
        0 => Ok(ReflectedValue::Bool(true)),
        _ => Err(ReflectError::UnknownField {
            type_path: "plugin_a::ProbeResource".to_string(),
            field_name: format!("#{field_slot}"),
        }),
    }
}

fn dummy_resource_write_fields_by_slot(
    world: &mut World,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    let mut changed = false;
    for (field_slot, value) in fields {
        if field_slot != 0 {
            return Err(ReflectError::UnknownField {
                type_path: "plugin_a::ProbeResource".to_string(),
                field_name: format!("#{field_slot}"),
            });
        }
        changed |= dummy_resource_read_field(world, "enabled")? != value;
    }
    Ok(changed)
}

fn dummy_resource_write_field_by_slot(
    world: &mut World,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_slot != 0 {
        return Err(ReflectError::UnknownField {
            type_path: "plugin_a::ProbeResource".to_string(),
            field_name: format!("#{field_slot}"),
        });
    }
    Ok(dummy_resource_read_field(world, "enabled")? != value)
}
