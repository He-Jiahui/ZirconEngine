//! 把插件组件描述符接入反射元数据，并为 World 的 JSON 动态组件提供操作表。
//! 回调按 World 当前目录取得字段；VM 路径按声明类型转换，兼容路径沿用 ScenePropertyValue 转换。

use crate::core::framework::scene::ComponentPropertyPath;
use crate::core::framework::scene::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::scene::{reflect::ReflectComponent, EntityId, World};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
};

/// 将插件组件描述符转换为可见反射模式；稳定字段 ID 使编辑器读写与动态组件 JSON 共用一份声明。
///
/// # 参数与返回值
///
/// `descriptor` 为共享借用；返回拥有型 JSON 组件元数据，保留属性顺序、可编辑标记和插件归属。
/// 字段 ID 取完整类型 ID 与属性名两个稳定键；完整路径、短名或插件 ID 无效时返回错误。
///
/// # 约束与生命周期
///
/// 这里只转换声明，不安装适配器或创建实例；字段元数据的完整准入由 `TypeRegistry` 完成。
/// `World::register_component_type` 将描述符、该候选和动态操作表一起预检后发布。
///
/// # 参考
///
/// `field_from_property_descriptor`、`reflect_component_for_dynamic_descriptor`、`World::register_component_type`。
pub fn registration_from_component_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<ReflectTypeRegistration, ReflectError> {
    let mut fields = Vec::with_capacity(descriptor.properties.len());
    for property in &descriptor.properties {
        fields.push(field_from_property_descriptor(
            &descriptor.type_id,
            property,
        ));
    }

    ReflectTypeRegistration::new(
        ReflectTypePath::new(
            descriptor.type_id.clone(),
            short_type_path(&descriptor.type_id).to_string(),
        )?,
        descriptor.display_name.clone(),
        ReflectTypeInfo::json_with_fields(fields),
        ReflectSerializationStrategy::Json,
    )
    .as_component()
    .with_serializable(true)
    .with_editor_visible(true)
    .with_remote_visible(true)
    .with_plugin_id(descriptor.plugin_id.clone())
}

/// 构造包含槽位读写、批量写入和暂存克隆的动态组件操作表。
///
/// `descriptor` 只借用到构建结束，表拥有其完整类型 ID；回调不捕获属性列表，而在调用时读取 World 的当前注册。
/// 调用方须将表与对应描述符/元数据一起安装，参见 `World::register_component_type` 与 `TypeRegistry` 的 VM 注册路径。
pub fn reflect_component_for_dynamic_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> ReflectComponent {
    ReflectComponent::new(
        descriptor.type_id.clone(),
        contains,
        read_field,
        write_field,
        remove,
    )
    .with_dense_field_slots(read_dense_slot, write_dense_slot)
    .with_dense_field_batch_write(write_dense_slots)
    .with_stage_clone(stage_clone)
}

/// 用类型身份键与规范属性名建立稳定字段 ID，并保留声明类型及可编辑标记。
///
/// 本模块的调用方以完整类型 ID 作为 `owner_key`；属性次序另决定当期槽位，改变次序不改变这里的稳定键。
fn field_from_property_descriptor(
    owner_key: &str,
    descriptor: &ComponentPropertyDescriptor,
) -> ReflectFieldInfo {
    ReflectFieldInfo::from_stable_keys(
        owner_key,
        &descriptor.name,
        descriptor.name.clone(),
        descriptor.value_type.clone(),
        ReflectEditorHint::None,
    )
    .with_editable(descriptor.editable)
}

fn short_type_path(type_path: &str) -> &str {
    if let Some((_, short)) = type_path.rsplit_once('.') {
        return short;
    }

    type_path
}

fn contains(world: &World, entity: EntityId, type_path: &str) -> bool {
    world.contains_entity(entity) && world.dynamic_component(entity, type_path).is_some()
}

/// 克隆源 JSON 组件并通过 `World::set_dynamic_component` 暂存到目标。
///
/// 目标须已恢复实体身份并准备对应动态类型目录；源值借用随源 World，返回前由目标接管克隆值。
/// 目标写入错误映射为暂存上下文的 `UnsupportedConversion`，参见 `World::stage_reflected_component_clone`。
fn stage_clone(
    source: &World,
    entity: EntityId,
    type_path: &str,
    target: &mut World,
) -> Result<(), ReflectError> {
    let value = ensure_dynamic_component(source, entity, type_path)?.clone();
    target
        .set_dynamic_component(entity, type_path, value)
        .map(|_| ())
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("dynamic component `{type_path}` staged clone"),
        })
}

fn read_field(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    let field = ensure_declared_field(registration, field_name)?;
    read_declared_field(world, entity, type_path, field)
}

fn read_dense_slot(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    let field = declared_field_by_slot(registration, field_slot)?;
    read_declared_field(world, entity, type_path, field)
}

/// 读取当前声明字段，先要求组件 JSON 是对象且实际包含该字段。
///
/// VM 分支按 `value_type_path` 解析为拥有型反射值；兼容分支经现有动态属性/ScenePropertyValue 转换。
/// 本路径不会因 schema 声明或默认值而自动补齐缺失 JSON 键。
fn read_declared_field(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field: &ReflectFieldInfo,
) -> Result<ReflectedValue, ReflectError> {
    let component = ensure_dynamic_component(world, entity, type_path)?;
    ensure_json_field_present(component, type_path, &field.name)?;
    if !world.is_vm_dynamic_type_path(type_path) {
        let property_path = dynamic_property_path(type_path, &field.name)?;
        let Some(value) = world.dynamic_component_property(entity, &property_path) else {
            return Err(ReflectError::UnsupportedConversion {
                source: format!("dynamic property `{property_path}` is missing or unsupported"),
                target: field.value_type_path.clone(),
            });
        };
        return super::reflected_from_scene_value(value);
    }

    let Some(object) = component.as_object() else {
        return Err(ReflectError::UnsupportedConversion {
            source: "dynamic component JSON is not an object".to_string(),
            target: type_path.to_string(),
        });
    };
    let Some(value) = object.get(&field.name) else {
        return Err(ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: field.name.clone(),
        });
    };
    super::dynamic_json::reflected_value_from_json(
        type_path,
        &field.name,
        &field.value_type_path,
        value,
    )
}

/// 从当前注册克隆字段元数据，再结束目录共享借用并进入可变 World 写入。
///
/// 按规范名解析的字段交给 `write_declared_field`；未知字段在写入前返回错误。
fn write_field(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let field = {
        let registration = world.type_registry().registration(type_path)?;
        ensure_declared_field(registration, field_name)?.clone()
    };
    write_declared_field(world, entity, type_path, &field, value)
}

/// 按当前注册的零基槽位克隆字段元数据，结束目录借用后交给 `write_declared_field`。
///
/// 槽位须与当前 schema 配对；本层不把它作为持久字段身份。
fn write_dense_slot(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let field = {
        let registration = world.type_registry().registration(type_path)?;
        declared_field_by_slot(registration, field_slot)?.clone()
    };
    write_declared_field(world, entity, type_path, &field, value)
}

/// 在独立 JSON 候选上按输入顺序完成字段转换，再调用一次 World 写入口。
///
/// 槽位存在且字段可编辑后，VM 分支检查声明类型并转 JSON；兼容分支经 ScenePropertyValue 转 JSON。
/// 字段或转换错误返回时尚未调用 `World::set_dynamic_component`；最终提交和变更标记由该 World 入口决定。
/// 重复槽位依次更新候选；无中间变化时直接返回 `false`，最终候选恢复原值时 World 也可返回 `false`。
/// 此回调本身不追加值树预算，门面准入见 `WorldReflection::reflect_write`。
fn write_dense_slots(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    let mut candidate = ensure_dynamic_component(world, entity, type_path)?.clone();
    let Some(object) = candidate.as_object_mut() else {
        return Err(ReflectError::UnsupportedConversion {
            source: "dynamic component JSON is not an object".to_string(),
            target: type_path.to_string(),
        });
    };

    let registration = world.type_registry().registration(type_path)?;
    let is_vm_type = world.is_vm_dynamic_type_path(type_path);
    let mut changed = false;
    for (field_slot, value) in fields {
        let field = declared_field_by_slot(registration, field_slot)?;
        if !field.editable {
            return Err(ReflectError::NonEditableField {
                type_path: type_path.to_string(),
                field_name: field.name.clone(),
            });
        }
        let json_value = if is_vm_type {
            super::type_registry::ensure_reflected_value_type(
                type_path,
                &field.name,
                &field.value_type_path,
                &value,
            )?;
            super::dynamic_json::json_value_from_reflected(value)?
        } else {
            let value = super::scene_value_from_reflected(value)?;
            crate::scene::world::json_from_scene_property_value(value).ok_or_else(|| {
                ReflectError::UnsupportedConversion {
                    source: "ScenePropertyValue".to_string(),
                    target: format!("dynamic component `{type_path}` batch update"),
                }
            })?
        };
        if object.get(&field.name) != Some(&json_value) {
            object.insert(field.name.clone(), json_value);
            changed = true;
        }
    }
    if !changed {
        return Ok(false);
    }

    world
        .set_dynamic_component(entity, type_path, candidate)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("dynamic component `{type_path}` batch update"),
        })
}

/// 检查可编辑性和现有组件后，按 World 的 VM 类型标识选择字段写入路径。
///
/// VM 分支先匹配声明反射类型、转换 JSON，再交给 World 的完整候选验证/提交入口；兼容分支沿用 ScenePropertyValue 转换。
/// World 错误映射为目标属性上下文的 `UnsupportedConversion`；值预算由上层门面另行检查。
fn write_declared_field(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field: &ReflectFieldInfo,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if !field.editable {
        return Err(ReflectError::NonEditableField {
            type_path: type_path.to_string(),
            field_name: field.name.clone(),
        });
    }
    ensure_dynamic_component(world, entity, type_path)?;
    let property_path = dynamic_property_path(type_path, &field.name)?;
    if !world.is_vm_dynamic_type_path(type_path) {
        let value = super::scene_value_from_reflected(value)?;
        return world
            .set_dynamic_component_property(entity, &property_path, value)
            .map_err(|error| ReflectError::UnsupportedConversion {
                source: error.to_string(),
                target: format!("dynamic property `{property_path}`"),
            });
    }

    super::type_registry::ensure_reflected_value_type(
        type_path,
        &field.name,
        &field.value_type_path,
        &value,
    )?;
    let value = super::dynamic_json::json_value_from_reflected(value)?;
    world
        .set_dynamic_component_json_property(entity, &property_path, value)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("dynamic JSON property `{type_path}.{}`", field.name),
        })
}

fn remove(world: &mut World, entity: EntityId, type_path: &str) -> Result<bool, ReflectError> {
    ensure_dynamic_component(world, entity, type_path)?;
    world
        .remove_dynamic_component(entity, type_path)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("dynamic component `{type_path}` removal"),
        })
}

/// 返回借用当前 World 的 JSON 组件，引用有效期随 World 共享借用。
///
/// 实体不存在与组件不存在分别返回 `MissingEntity` 和 `MissingComponent`，类型路径用于组件查找与错误上下文。
fn ensure_dynamic_component<'a>(
    world: &'a World,
    entity: EntityId,
    type_path: &str,
) -> Result<&'a serde_json::Value, ReflectError> {
    if !world.contains_entity(entity) {
        return Err(ReflectError::MissingEntity { entity });
    }

    let Some(component) = world.dynamic_component(entity, type_path) else {
        return Err(ReflectError::MissingComponent {
            entity,
            type_path: type_path.to_string(),
        });
    };

    Ok(component)
}

/// 在当前注册中按规范字段名精确查找，返回随注册借用有效的字段元数据。
///
/// 此内部按名回调不解析别名；稳定 ID 和兼容身份先由 catalog 解析为当前槽位。
fn ensure_declared_field<'a>(
    registration: &'a ReflectTypeRegistration,
    field_name: &str,
) -> Result<&'a ReflectFieldInfo, ReflectError> {
    for field in &registration.type_info.fields {
        if field.name == field_name {
            return Ok(field);
        }
    }

    Err(ReflectError::UnknownField {
        type_path: registration.type_path.type_path().to_string(),
        field_name: field_name.to_string(),
    })
}

fn ensure_json_field_present(
    component: &serde_json::Value,
    type_path: &str,
    field_name: &str,
) -> Result<(), ReflectError> {
    let Some(object) = component.as_object() else {
        return Err(ReflectError::UnsupportedConversion {
            source: "dynamic component JSON is not an object".to_string(),
            target: type_path.to_string(),
        });
    };
    if object.contains_key(field_name) {
        return Ok(());
    }
    Err(ReflectError::UnknownField {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
    })
}

#[cfg(test)]
#[path = "tests/dynamic_component.rs"]
mod tests;

/// 把当前注册的零基槽位解释为字段序列索引；返回引用借用该注册。
///
/// 越界以 `#<slot>` 上下文返回 `UnknownField`；schema 顺序变化后不能继续沿用旧槽位。
fn declared_field_by_slot(
    registration: &ReflectTypeRegistration,
    field_slot: u32,
) -> Result<&ReflectFieldInfo, ReflectError> {
    let Some(field) = registration.type_info.fields.get(field_slot as usize) else {
        return Err(ReflectError::UnknownField {
            type_path: registration.type_path.type_path().to_string(),
            field_name: format!("#{field_slot}"),
        });
    };
    Ok(field)
}

/// 构造完整组件 ID 加单个规范字段段的结构化属性路径。
///
/// 直接保存两个结构化部分，供 World setter 使用；构造错误映射为 `InvalidRegistration`，参见 `ComponentPropertyPath::new`。
fn dynamic_property_path(
    type_path: &str,
    field_name: &str,
) -> Result<ComponentPropertyPath, ReflectError> {
    let mut property_segments = Vec::with_capacity(1);
    property_segments.push(field_name.to_string());

    ComponentPropertyPath::new(type_path.to_string(), property_segments).map_err(|error| {
        ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: error.to_string(),
        }
    })
}
