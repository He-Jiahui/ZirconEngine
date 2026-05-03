use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) fn parse_component<T>(value: &Value) -> T
where
    T: Default + DeserializeOwned,
{
    serde_json::from_value(normalize_scene_property_json(value)).unwrap_or_default()
}

fn normalize_scene_property_json(value: &Value) -> Value {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("resource") => object
            .get("resource")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
        Value::Object(object) if object.len() == 1 && object.contains_key("entity") => {
            object.get("entity").cloned().unwrap_or(Value::Null)
        }
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), normalize_scene_property_json(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(normalize_scene_property_json)
                .collect::<Vec<_>>(),
        ),
        value => value.clone(),
    }
}
