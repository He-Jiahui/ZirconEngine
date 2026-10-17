//! 把 `ZrReflect` 元数据与类型化 ECS 回调组成 Runtime 注册候选。
//! 字段写入先作用于 `T::clone` 候选，再进入 World writer；LocalTransform 与 Mobility 使用各自的受检写入口。

use std::any::Any;

use zircon_runtime_interface::reflect::{
    ReflectError, ReflectTypeRegistration, ReflectedValue, ZrReflect,
};

use crate::scene::components::{LocalTransform, Mobility};
use crate::scene::ecs::Component;
use crate::scene::{EntityId, SceneError, World};

use crate::scene::reflect::{ReflectComponent, RuntimeTypeRegistration};

/// 为可克隆 ECS 组件注册读写及预检克隆适配器；调用者须在 World 发布前安装唯一的类型路径。
///
/// # 使用时机与返回值
///
/// 为 `T: Component + ZrReflect + Clone` 构造组件角色候选，包含按名/槽位读写、批量写入、移除及预检克隆。
/// 元数据构造或组件角色检查失败时返回错误；成功不会自动发布目录，调用方再交给 `TypeRegistry::register`。
///
/// # 约束与生命周期
///
/// 槽位顺序由 `T` 的反射实现与元数据配对；派生宏从同一字段序列生成二者。
/// 回调按调用参数借用 World，不保存组件引用。候选克隆遵循 `T::clone`，共享资源的隔离程度由 `T` 决定。
///
/// # 参考
///
/// `ZrReflect`、`finish_component_registration`、`commit_component`；内建反射注册与本模块测试。
pub fn derived_component_registration<T>() -> Result<RuntimeTypeRegistration, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let registration = T::reflect_type_registration()?;
    let type_path = registration.type_path.type_path().to_string();
    finish_component_registration(
        registration,
        ReflectComponent::new(
            type_path,
            contains::<T>,
            read_field::<T>,
            write_field::<T>,
            remove::<T>,
        )
        .with_dense_field_slots(read_field_by_slot::<T>, write_field_by_slot::<T>)
        .with_dense_field_batch_write(write_fields_by_slot::<T>)
        .with_stage_clone(stage_clone::<T>),
    )
}

/// 把 `T` 的反射元数据与调用方提供的组件表组成候选。
///
/// # 参数与返回值
///
/// `component` 的所有权移入候选；`T` 只须提供 `ZrReflect` 元数据。
/// 成功返回含组件表、无资源表的注册；元数据构造或组件角色检查失败时返回错误。
///
/// # 约束
///
/// 本工厂只检查组件角色，不校验回调与 `T` 的存储、路径或槽位是否相符，调用方须完成配对。
/// 候选的完整准入与发布另由 `TypeRegistry` 执行；受保护的内建类型据此安装专用 World 访问器。
///
/// # 参考
///
/// `finish_component_registration`；内建 Hierarchy、ActiveInHierarchy 的注册工厂。
pub fn derived_component_registration_with_adapter<T>(
    component: ReflectComponent,
) -> Result<RuntimeTypeRegistration, ReflectError>
where
    T: ZrReflect,
{
    let registration = T::reflect_type_registration()?;
    finish_component_registration(registration, component)
}

fn finish_component_registration(
    registration: ReflectTypeRegistration,
    component: ReflectComponent,
) -> Result<RuntimeTypeRegistration, ReflectError> {
    validate_component_registration(&registration)?;
    Ok(RuntimeTypeRegistration {
        registration,
        component: Some(component),
        resource: None,
    })
}

fn validate_component_registration(
    registration: &ReflectTypeRegistration,
) -> Result<(), ReflectError> {
    if registration.is_component() {
        return Ok(());
    }
    Err(ReflectError::InvalidRegistration {
        type_path: registration.type_path.type_path().to_string(),
        reason: "derived component adapters require component-only registrations".to_string(),
    })
}

fn contains<T>(world: &World, entity: EntityId, _type_path: &str) -> bool
where
    T: Component + ZrReflect + Clone,
{
    world.get::<T>(entity).is_some()
}

/// 把源组件的 `T::clone` 结果移入目标预检 World。
///
/// 目标须先恢复同一稳定实体身份。LocalTransform/Mobility 通过预检 authored clone 入口暂存，其余类型走 `World::insert`。
/// 源组件借用仅用于克隆；World 错误转为带暂存上下文的反射错误，参见 `World::stage_reflected_component_clone`。
fn stage_clone<T>(
    source: &World,
    entity: EntityId,
    type_path: &str,
    target: &mut World,
) -> Result<(), ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let value = component::<T>(source, entity, type_path)?.clone();
    let result =
        if (&value as &dyn Any).is::<LocalTransform>() || (&value as &dyn Any).is::<Mobility>() {
            target.stage_prevalidated_authored_clone(entity, value)
        } else {
            target.insert(entity, value).map(|_| ())
        };
    result.map_err(|error| error.into_reflect_error(format!("{type_path} staged component clone")))
}

fn read_field<T>(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    component::<T>(world, entity, type_path)?.read_reflected_field(field_name)
}

/// 在组件克隆候选上按名写入，字段转换成功且报告变化后才进入 `commit_component`。
///
/// `value` 由反射实现消费；错误或无变化在调用 World writer 前返回。`Clone` 的内部共享行为由 `T` 负责。
fn write_field<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let mut next = component::<T>(world, entity, type_path)?.clone();
    if !next.write_reflected_field(field_name, value)? {
        return Ok(false);
    }
    commit_component(world, entity, next, format!("{type_path}.{field_name}"))
}

fn read_field_by_slot<T>(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    component::<T>(world, entity, type_path)?.read_reflected_field_by_slot(field_slot)
}

/// 在克隆候选上消费当前槽位的字段值；槽位须对应 `T` 的当期反射字段顺序。
///
/// 字段错误或无变化时不调用 World writer；成功且报告变化后交给 `commit_component`。
fn write_field_by_slot<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let mut next = component::<T>(world, entity, type_path)?.clone();
    if !next.write_reflected_field_by_slot(field_slot, value)? {
        return Ok(false);
    }
    commit_component(world, entity, next, format!("{type_path}.#{field_slot}"))
}

/// 在同一个组件克隆候选上按输入顺序消费整批字段值。
///
/// 各槽位写入成功后才进入一次 `commit_component`；中途字段错误返回时尚未调用 World writer。
/// 重复槽位依次作用于候选，变化标记为各字段结果的累积；最终返回值采用提交入口的结果。
/// 空列表或所有字段均报告无变化时返回 `false`；候选的共享资源语义仍由 `T::clone` 决定。
fn write_fields_by_slot<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let mut next = component::<T>(world, entity, type_path)?.clone();
    let mut changed = false;
    for (field_slot, value) in fields {
        changed |= next.write_reflected_field_by_slot(field_slot, value)?;
    }
    if !changed {
        return Ok(false);
    }
    commit_component(world, entity, next, format!("{type_path}.#batch"))
}

/// 把已经完成字段转换的拥有型候选交给对应 World 写入口。
///
/// LocalTransform 使用 `World::update_transform`，Mobility 使用 `World::set_mobility`，保留领域验证及各自的变化标记。
/// 其余类型通过 `World::insert` 提交，插入成功返回 `true`；错误按 `target` 上下文映射。
/// 因此本层不另做通用的最终值相等比较，参见上述 World writer。
fn commit_component<T>(
    world: &mut World,
    entity: EntityId,
    next: T,
    target: String,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    // Reflected authored values retain ZrReflect's field checks, then use Scene's checked writer.
    if let Some(local) = (&next as &dyn Any).downcast_ref::<LocalTransform>() {
        return match world.update_transform(entity, local.transform) {
            Ok(changed) => Ok(changed),
            Err(error) => Err(error.into_reflect_error(target)),
        };
    }
    if let Some(mobility) = (&next as &dyn Any).downcast_ref::<Mobility>() {
        return match world.set_mobility(entity, *mobility) {
            Ok(changed) => Ok(changed),
            Err(error) => Err(error.into_reflect_error(target)),
        };
    }
    match world.insert(entity, next) {
        Ok(_) => Ok(true),
        Err(error) => Err(error.into_reflect_error(target)),
    }
}

/// 检查实体与组件后，通过 `World::remove` 移除类型化组件。
///
/// 成功取出组件返回 `true`；受保护 authored/derived 类型返回 `NonRemovableComponent`。
/// 生命周期目标改变错误保留对应映射，其余失败按本适配器现有规则归为组件缺失。
fn remove<T>(world: &mut World, entity: EntityId, type_path: &str) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    ensure_entity(world, entity)?;
    if world.get::<T>(entity).is_none() {
        return Err(missing_component(entity, type_path));
    }
    match world.remove::<T>(entity) {
        Ok(Some(_)) => Ok(true),
        Err(SceneError::ProtectedDerivedComponentMutation { .. })
        | Err(SceneError::ProtectedAuthoredComponentMutation { .. }) => {
            Err(ReflectError::NonRemovableComponent {
                type_path: type_path.to_string(),
            })
        }
        Ok(None) => Err(missing_component(entity, type_path)),
        Err(error) if error.lifecycle_target_change().is_some() => {
            Err(error.into_reflect_error(type_path.to_owned()))
        }
        Err(_) => Err(missing_component(entity, type_path)),
    }
}

/// 借用指定 World 中的类型化组件，返回引用的有效期随该 World 共享借用。
///
/// 先区分实体不存在与组件不存在；`type_path` 只用于缺失错误上下文，实际存储查找按 `T`。
fn component<'a, T>(
    world: &'a World,
    entity: EntityId,
    type_path: &str,
) -> Result<&'a T, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    ensure_entity(world, entity)?;
    let Some(component) = world.get::<T>(entity) else {
        return Err(missing_component(entity, type_path));
    };
    Ok(component)
}

fn ensure_entity(world: &World, entity: EntityId) -> Result<(), ReflectError> {
    if world.contains_entity(entity) {
        return Ok(());
    }
    Err(ReflectError::MissingEntity { entity })
}

fn missing_component(entity: EntityId, type_path: &str) -> ReflectError {
    ReflectError::MissingComponent {
        entity,
        type_path: type_path.to_string(),
    }
}
