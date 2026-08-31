use std::collections::HashMap;

use serde_json::{Map, Number, Value};
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue, ReflectedValue,
};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::dynamic_scene::{DynamicSceneError, EntityRemap};
use crate::scene::World;

use super::remap::remap_json_entity_objects;

pub(in crate::scene::dynamic_scene) fn reflected_fields_to_json_object(
    world: &World,
    type_path: &str,
    schema_fields: &[ReflectFieldInfo],
    fields: &[ReflectFieldValue],
    remap: &EntityRemap,
) -> Result<Value, DynamicSceneError> {
    let mut object = Map::with_capacity(fields.len());
    let mut seen_slots = vec![false; schema_fields.len()];
    let mut next_schema_slot = 0usize;
    for field in fields {
        let field_slot = if schema_fields
            .get(next_schema_slot)
            .is_some_and(|schema| schema.id == field.field_id)
        {
            next_schema_slot
        } else {
            world
                .type_registry()
                .resolve_field_slot_by_id(type_path, field.field_id)? as usize
        };
        let schema = schema_fields
            .get(field_slot)
            .filter(|schema| schema.id == field.field_id)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field.field_id.to_string(),
            })?;
        if std::mem::replace(&mut seen_slots[field_slot], true) {
            return Err(ReflectError::InvalidValue {
                type_path: type_path.to_string(),
                field_name: field.field_id.to_string(),
                reason: "duplicate stable field identity in dynamic scene payload".to_string(),
            }
            .into());
        }
        next_schema_slot = field_slot.saturating_add(1);
        object.insert(
            schema.name.clone(),
            reflected_value_to_json(&field.value, remap, &schema.name)?,
        );
    }
    Ok(Value::Object(object))
}

pub(in crate::scene::dynamic_scene) fn descriptor_fields_to_json_object(
    descriptor: &ComponentTypeDescriptor,
    fields: &[ReflectFieldValue],
    remap: &EntityRemap,
) -> Result<Value, DynamicSceneError> {
    let mut names_by_id = HashMap::with_capacity(descriptor.properties.len());
    for property in &descriptor.properties {
        names_by_id.insert(
            ReflectFieldId::from_stable_keys(&descriptor.type_id, &property.name),
            property.name.as_str(),
        );
    }

    let mut object = Map::with_capacity(fields.len());
    for field in fields {
        let Some(field_name) = names_by_id.remove(&field.field_id) else {
            return Err(ReflectError::UnknownField {
                type_path: descriptor.type_id.clone(),
                field_name: field.field_id.to_string(),
            }
            .into());
        };
        object.insert(
            field_name.to_string(),
            reflected_value_to_json(&field.value, remap, field_name)?,
        );
    }
    Ok(Value::Object(object))
}

fn reflected_value_to_json(
    value: &ReflectedValue,
    remap: &EntityRemap,
    context: &str,
) -> Result<Value, DynamicSceneError> {
    Ok(match value {
        ReflectedValue::Null => Value::Null,
        ReflectedValue::Bool(value) => Value::Bool(*value),
        ReflectedValue::Integer(value) => Value::Number((*value).into()),
        ReflectedValue::Unsigned(value) => Value::Number((*value).into()),
        ReflectedValue::Scalar(value) => Number::from_f64(*value as f64)
            .map(Value::Number)
            .ok_or_else(|| unsupported_value(context, "finite f32"))?,
        ReflectedValue::String(value) | ReflectedValue::Enum(value) => Value::String(value.clone()),
        ReflectedValue::Vec2(values) => float_array_to_json(values, context)?,
        ReflectedValue::Vec3(values) => float_array_to_json(values, context)?,
        ReflectedValue::Vec4(values) | ReflectedValue::Quaternion(values) => {
            float_array_to_json(values, context)?
        }
        ReflectedValue::Entity(entity) => Value::Object(Map::from_iter([(
            "entity".to_string(),
            entity
                .map(|entity| Value::Number(Number::from(remap.get(entity).unwrap_or(entity))))
                .unwrap_or(Value::Null),
        )])),
        ReflectedValue::Resource(value) => Value::Object(Map::from_iter([(
            "resource".to_string(),
            Value::String(value.clone()),
        )])),
        ReflectedValue::List(values) => Value::Array(
            values
                .iter()
                .map(|value| reflected_value_to_json(value, remap, context))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ReflectedValue::Map(values) => values
            .iter()
            .map(|(key, value)| Ok((key.clone(), reflected_value_to_json(value, remap, context)?)))
            .collect::<Result<Map<_, _>, DynamicSceneError>>()
            .map(Value::Object)?,
        ReflectedValue::Json(value) => remap_json_entity_objects(value, remap),
    })
}

fn float_array_to_json<const N: usize>(
    values: &[f32; N],
    context: &str,
) -> Result<Value, DynamicSceneError> {
    values
        .iter()
        .map(|value| {
            Number::from_f64(*value as f64)
                .map(Value::Number)
                .ok_or_else(|| unsupported_value(context, "finite f32"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Value::Array)
}

fn unsupported_value(context: &str, type_name: &'static str) -> DynamicSceneError {
    DynamicSceneError::UnsupportedValue {
        context: context.to_string(),
        type_name,
    }
}
