use std::collections::BTreeMap;

use serde_json::{Map, Number, Value};
use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectedValue};

use super::{DynamicSceneError, EntityRemap};

pub(super) fn remap_reflected_value(
    value: &ReflectedValue,
    remap: &EntityRemap,
) -> Result<ReflectedValue, DynamicSceneError> {
    Ok(match value {
        ReflectedValue::Entity(Some(entity)) => {
            ReflectedValue::Entity(Some(remap.get(*entity).unwrap_or(*entity)))
        }
        ReflectedValue::List(values) => ReflectedValue::List(
            values
                .iter()
                .map(|value| remap_reflected_value(value, remap))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ReflectedValue::Map(values) => ReflectedValue::Map(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), remap_reflected_value(value, remap)?)))
                .collect::<Result<BTreeMap<_, _>, DynamicSceneError>>()?,
        ),
        ReflectedValue::Json(value) => {
            ReflectedValue::Json(remap_json_entity_objects(value, remap))
        }
        value => value.clone(),
    })
}

pub(super) fn reflected_fields_to_json_object(
    fields: &[ReflectFieldValue],
    remap: &EntityRemap,
) -> Result<Value, DynamicSceneError> {
    fields
        .iter()
        .map(|field| {
            Ok((
                field.field_name.clone(),
                reflected_value_to_json(&field.value, remap, &field.field_name)?,
            ))
        })
        .collect::<Result<Map<_, _>, DynamicSceneError>>()
        .map(Value::Object)
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

fn remap_json_entity_objects(value: &Value, remap: &EntityRemap) -> Value {
    match value {
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| remap_json_entity_objects(value, remap))
                .collect(),
        ),
        Value::Object(object) if object.len() == 1 && object.contains_key("entity") => {
            Value::Object(Map::from_iter([(
                "entity".to_string(),
                remap_json_entity_value(&object["entity"], remap),
            )]))
        }
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), remap_json_entity_objects(value, remap)))
                .collect(),
        ),
        value => value.clone(),
    }
}

fn remap_json_entity_value(value: &Value, remap: &EntityRemap) -> Value {
    match value {
        Value::Number(number) => number
            .as_u64()
            .map(|entity| Number::from(remap.get(entity).unwrap_or(entity)))
            .map(Value::Number)
            .unwrap_or_else(|| value.clone()),
        Value::Null => Value::Null,
        value => value.clone(),
    }
}

fn unsupported_value(context: &str, type_name: &'static str) -> DynamicSceneError {
    DynamicSceneError::UnsupportedValue {
        context: context.to_string(),
        type_name,
    }
}
