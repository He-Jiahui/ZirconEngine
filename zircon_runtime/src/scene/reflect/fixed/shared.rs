use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
};

use crate::scene::{reflect::ReflectComponent, reflect::RuntimeTypeRegistration, EntityId, World};

pub(super) fn component_registration(
    type_path: &'static str,
    short_type_path: &'static str,
    fields: Vec<ReflectFieldInfo>,
    adapter: ReflectComponent,
) -> RuntimeTypeRegistration {
    RuntimeTypeRegistration {
        registration: ReflectTypeRegistration::new(
            ReflectTypePath::new(type_path, short_type_path)
                .expect("fixed component reflection type paths must be valid"),
            short_type_path,
            ReflectTypeInfo::struct_with_fields(fields),
            ReflectSerializationStrategy::Value,
        )
        .as_component(),
        component: Some(adapter),
        resource: None,
    }
}

pub(super) fn field(
    name: &'static str,
    value_type_path: &'static str,
    hint: ReflectEditorHint,
) -> ReflectFieldInfo {
    ReflectFieldInfo::new(name, value_type_path, hint)
}

pub(super) fn readonly_field(
    name: &'static str,
    value_type_path: &'static str,
    hint: ReflectEditorHint,
) -> ReflectFieldInfo {
    field(name, value_type_path, hint).with_editable(false)
}

pub(super) fn ensure_entity(world: &World, entity: EntityId) -> Result<(), ReflectError> {
    if world.contains_entity(entity) {
        return Ok(());
    }

    Err(ReflectError::MissingEntity { entity })
}

pub(super) fn get_component<'a, T>(
    world: &'a World,
    entity: EntityId,
    type_path: &'static str,
) -> Result<&'a T, ReflectError>
where
    T: crate::scene::ecs::Component,
{
    ensure_entity(world, entity)?;
    let Some(component) = world.get::<T>(entity) else {
        return Err(missing_component(entity, type_path));
    };

    Ok(component)
}

pub(super) fn get_component_mut<'a, T>(
    world: &'a mut World,
    entity: EntityId,
    type_path: &'static str,
) -> Result<&'a mut T, ReflectError>
where
    T: crate::scene::ecs::Component,
{
    ensure_entity(world, entity)?;
    let Some(component) = world.get_mut::<T>(entity) else {
        return Err(missing_component(entity, type_path));
    };

    Ok(component)
}

pub(super) fn ensure_component<T>(
    world: &World,
    entity: EntityId,
    type_path: &'static str,
) -> Result<(), ReflectError>
where
    T: crate::scene::ecs::Component,
{
    ensure_entity(world, entity)?;
    if world.get::<T>(entity).is_none() {
        return Err(missing_component(entity, type_path));
    }

    Ok(())
}

pub(super) fn missing_component(entity: EntityId, type_path: &'static str) -> ReflectError {
    ReflectError::MissingComponent {
        entity,
        type_path: type_path.to_string(),
    }
}

pub(super) fn unknown_field(type_path: &'static str, field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
    }
}

pub(super) fn non_editable_field(type_path: &'static str, field_name: &str) -> ReflectError {
    ReflectError::NonEditableField {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
    }
}

pub(super) fn field_target(type_path: &'static str, field_name: &str) -> String {
    let mut target = String::with_capacity(type_path.len() + 1 + field_name.len());
    target.push_str(type_path);
    target.push('.');
    target.push_str(field_name);
    target
}

pub(super) fn type_mismatch(
    type_path: &'static str,
    field_name: &str,
    expected: &'static str,
    actual: &ReflectedValue,
) -> ReflectError {
    ReflectError::TypeMismatch {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: actual.type_name().to_string(),
    }
}

fn invalid_value(
    type_path: &'static str,
    field_name: &str,
    expected: &'static str,
    actual: &'static str,
) -> ReflectError {
    ReflectError::TypeMismatch {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}

pub(super) fn expect_bool(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    match value {
        ReflectedValue::Bool(value) => Ok(value),
        value => Err(type_mismatch(type_path, field_name, "Bool", &value)),
    }
}

pub(super) fn expect_i32(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<i32, ReflectError> {
    match value {
        ReflectedValue::Integer(value) => match i32::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(invalid_value(
                type_path,
                field_name,
                "i32 Integer",
                "out-of-range Integer",
            )),
        },
        value => Err(type_mismatch(type_path, field_name, "Integer", &value)),
    }
}

pub(super) fn expect_scalar(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<f32, ReflectError> {
    match value {
        ReflectedValue::Scalar(value) if value.is_finite() => Ok(value),
        ReflectedValue::Scalar(_) => Err(invalid_value(
            type_path,
            field_name,
            "finite Scalar",
            "non-finite Scalar",
        )),
        value => Err(type_mismatch(type_path, field_name, "Scalar", &value)),
    }
}

pub(super) fn expect_string(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<String, ReflectError> {
    match value {
        ReflectedValue::String(value) => Ok(value),
        value => Err(type_mismatch(type_path, field_name, "String", &value)),
    }
}

pub(super) fn expect_vec3(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<[f32; 3], ReflectError> {
    match value {
        ReflectedValue::Vec3(value) if vec3_components_are_finite(&value) => Ok(value),
        ReflectedValue::Vec3(_) => Err(invalid_value(
            type_path,
            field_name,
            "finite Vec3",
            "non-finite Vec3",
        )),
        value => Err(type_mismatch(type_path, field_name, "Vec3", &value)),
    }
}

pub(super) fn expect_vec2(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<[f32; 2], ReflectError> {
    match value {
        ReflectedValue::Vec2(value) if vec2_components_are_finite(&value) => Ok(value),
        ReflectedValue::Vec2(_) => Err(invalid_value(
            type_path,
            field_name,
            "finite Vec2",
            "non-finite Vec2",
        )),
        value => Err(type_mismatch(type_path, field_name, "Vec2", &value)),
    }
}

pub(super) fn expect_vec4(
    type_path: &'static str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<[f32; 4], ReflectError> {
    match value {
        ReflectedValue::Vec4(value) if vec4_components_are_finite(&value) => Ok(value),
        ReflectedValue::Vec4(_) => Err(invalid_value(
            type_path,
            field_name,
            "finite Vec4",
            "non-finite Vec4",
        )),
        value => Err(type_mismatch(type_path, field_name, "Vec4", &value)),
    }
}

fn vec2_components_are_finite(value: &[f32; 2]) -> bool {
    value[0].is_finite() && value[1].is_finite()
}

fn vec3_components_are_finite(value: &[f32; 3]) -> bool {
    value[0].is_finite() && value[1].is_finite() && value[2].is_finite()
}

fn vec4_components_are_finite(value: &[f32; 4]) -> bool {
    value[0].is_finite() && value[1].is_finite() && value[2].is_finite() && value[3].is_finite()
}

pub(super) fn remove_component<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &'static str,
) -> Result<bool, ReflectError>
where
    T: crate::scene::ecs::Component,
{
    ensure_entity(world, entity)?;
    if world.get::<T>(entity).is_none() {
        return Err(missing_component(entity, type_path));
    }
    match world.remove::<T>(entity) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Err(missing_component(entity, type_path)),
        Err(_) => Err(missing_component(entity, type_path)),
    }
}
