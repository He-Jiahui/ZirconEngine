use std::collections::BTreeMap;

use serde_json::{Map, Value};
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::declared_value_type::DeclaredValueType;

pub(in crate::scene) fn ensure_json_value_type(
    type_path: &str,
    field_name: &str,
    expected: &str,
    value: &Value,
) -> Result<(), ReflectError> {
    let reflected = reflected_value_from_json(type_path, field_name, expected, value)?;
    super::validate_reflected_value(type_path, field_name, &reflected)
}

pub(in crate::scene) fn reflected_value_from_json(
    type_path: &str,
    field_name: &str,
    expected: &str,
    value: &Value,
) -> Result<ReflectedValue, ReflectError> {
    let declared =
        DeclaredValueType::parse(expected).map_err(|reason| ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: format!("reflected field `{field_name}` {reason}"),
        })?;
    let converted = reflected_value_from_declared_json(type_path, field_name, &declared, value)?;
    converted.ok_or_else(|| ReflectError::TypeMismatch {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: json_value_type_name(value).to_string(),
    })
}

fn reflected_value_from_declared_json(
    type_path: &str,
    field_name: &str,
    declared: &DeclaredValueType,
    value: &Value,
) -> Result<Option<ReflectedValue>, ReflectError> {
    let converted = match declared {
        DeclaredValueType::Null => value.is_null().then_some(ReflectedValue::Null),
        DeclaredValueType::Bool => value.as_bool().map(ReflectedValue::Bool),
        DeclaredValueType::Integer => value.as_i64().map(ReflectedValue::Integer),
        DeclaredValueType::Unsigned => value.as_u64().map(ReflectedValue::Unsigned),
        DeclaredValueType::Scalar => json_f32(value).map(ReflectedValue::Scalar),
        DeclaredValueType::String => value
            .as_str()
            .map(|value| ReflectedValue::String(value.to_string())),
        DeclaredValueType::Enum => value
            .as_str()
            .map(|value| ReflectedValue::Enum(value.to_string())),
        DeclaredValueType::Vec2 => json_vector::<2>(value).map(ReflectedValue::Vec2),
        DeclaredValueType::Vec3 => json_vector::<3>(value).map(ReflectedValue::Vec3),
        DeclaredValueType::Vec4 => json_vector::<4>(value).map(ReflectedValue::Vec4),
        DeclaredValueType::Quaternion => json_vector::<4>(value).map(ReflectedValue::Quaternion),
        DeclaredValueType::Entity => reflected_entity_from_json(value),
        DeclaredValueType::Resource => reflected_resource_from_json(value),
        DeclaredValueType::Json => Some(ReflectedValue::Json(value.clone())),
        DeclaredValueType::List(item_type) => {
            reflected_list_from_json(type_path, field_name, item_type, value)?
        }
        DeclaredValueType::Map(value_type) => {
            reflected_map_from_json(type_path, field_name, value_type, value)?
        }
    };
    Ok(converted)
}

pub(in crate::scene) fn json_value_from_reflected(
    value: ReflectedValue,
) -> Result<Value, ReflectError> {
    let value = match value {
        ReflectedValue::Null => Value::Null,
        ReflectedValue::Bool(value) => Value::Bool(value),
        ReflectedValue::Integer(value) => Value::Number(value.into()),
        ReflectedValue::Unsigned(value) => Value::Number(value.into()),
        ReflectedValue::Scalar(value) => Value::Number(reflected_json_number(value)?),
        ReflectedValue::String(value) | ReflectedValue::Enum(value) => Value::String(value),
        ReflectedValue::Vec2(values) => reflected_vector_json(values)?,
        ReflectedValue::Vec3(values) => reflected_vector_json(values)?,
        ReflectedValue::Vec4(values) | ReflectedValue::Quaternion(values) => {
            reflected_vector_json(values)?
        }
        ReflectedValue::Entity(value) => {
            let mut object = Map::with_capacity(1);
            object.insert(
                "entity".to_string(),
                value.map_or(Value::Null, |entity| Value::Number(entity.into())),
            );
            Value::Object(object)
        }
        ReflectedValue::Resource(value) => {
            let mut object = Map::with_capacity(1);
            object.insert("resource".to_string(), Value::String(value));
            Value::Object(object)
        }
        ReflectedValue::List(values) => Value::Array(
            values
                .into_iter()
                .map(json_value_from_reflected)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ReflectedValue::Map(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, json_value_from_reflected(value)?)))
                .collect::<Result<Map<_, _>, ReflectError>>()?,
        ),
        ReflectedValue::Json(value) => value,
    };
    Ok(value)
}

fn reflected_list_from_json(
    type_path: &str,
    field_name: &str,
    item_type: &DeclaredValueType,
    value: &Value,
) -> Result<Option<ReflectedValue>, ReflectError> {
    let Some(values) = value.as_array() else {
        return Ok(None);
    };
    let values = values
        .iter()
        .map(|value| {
            reflected_value_from_declared_json(type_path, field_name, item_type, value)?.ok_or_else(
                || ReflectError::TypeMismatch {
                    type_path: type_path.to_string(),
                    field_name: field_name.to_string(),
                    expected: item_type.to_string(),
                    actual: json_value_type_name(value).to_string(),
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(ReflectedValue::List(values)))
}

fn reflected_map_from_json(
    type_path: &str,
    field_name: &str,
    value_type: &DeclaredValueType,
    value: &Value,
) -> Result<Option<ReflectedValue>, ReflectError> {
    let Some(values) = value.as_object() else {
        return Ok(None);
    };
    let values = values
        .iter()
        .map(|(key, value)| {
            let value =
                reflected_value_from_declared_json(type_path, field_name, value_type, value)?
                    .ok_or_else(|| ReflectError::TypeMismatch {
                        type_path: type_path.to_string(),
                        field_name: field_name.to_string(),
                        expected: value_type.to_string(),
                        actual: json_value_type_name(value).to_string(),
                    })?;
            Ok((key.clone(), value))
        })
        .collect::<Result<BTreeMap<_, _>, ReflectError>>()?;
    Ok(Some(ReflectedValue::Map(values)))
}

fn reflected_entity_from_json(value: &Value) -> Option<ReflectedValue> {
    if value.is_null() {
        return Some(ReflectedValue::Entity(None));
    }
    if let Some(entity) = value.as_u64() {
        return Some(ReflectedValue::Entity(Some(entity)));
    }
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    let entity = object.get("entity")?;
    match entity {
        Value::Null => Some(ReflectedValue::Entity(None)),
        value => value
            .as_u64()
            .map(|entity| ReflectedValue::Entity(Some(entity))),
    }
}

fn reflected_resource_from_json(value: &Value) -> Option<ReflectedValue> {
    if let Some(resource) = value.as_str() {
        return Some(ReflectedValue::Resource(resource.to_string()));
    }
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    let resource = object.get("resource")?.as_str()?;
    Some(ReflectedValue::Resource(resource.to_string()))
}

fn json_vector<const N: usize>(value: &Value) -> Option<[f32; N]> {
    let values = value.as_array()?;
    if values.len() != N {
        return None;
    }
    let mut result = [0.0; N];
    for (index, value) in values.iter().enumerate() {
        result[index] = json_f32(value)?;
    }
    Some(result)
}

fn json_f32(value: &Value) -> Option<f32> {
    let value = value.as_f64()? as f32;
    value.is_finite().then_some(value)
}

fn reflected_vector_json<const N: usize>(values: [f32; N]) -> Result<Value, ReflectError> {
    let values = values
        .into_iter()
        .map(reflected_json_number)
        .map(|number| number.map(Value::Number))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(values))
}

fn reflected_json_number(value: f32) -> Result<serde_json::Number, ReflectError> {
    let number = value
        .is_finite()
        .then(|| value.to_string().parse::<f64>().ok())
        .flatten()
        .and_then(serde_json::Number::from_f64);
    number.ok_or_else(|| ReflectError::UnsupportedConversion {
        source: "non-finite reflected scalar".to_string(),
        target: "dynamic JSON number".to_string(),
    })
}

fn json_value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "JsonNull",
        Value::Bool(_) => "JsonBool",
        Value::Number(_) => "JsonNumber",
        Value::String(_) => "JsonString",
        Value::Array(_) => "JsonArray",
        Value::Object(_) => "JsonObject",
    }
}
