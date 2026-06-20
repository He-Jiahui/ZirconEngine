use std::collections::BTreeMap;

use serde_json::{Map, Number, Value};
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::scene::dynamic_scene::{DynamicSceneError, EntityRemap};

pub(in crate::scene::dynamic_scene) fn remap_reflected_value(
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

pub(super) fn remap_json_entity_objects(value: &Value, remap: &EntityRemap) -> Value {
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
