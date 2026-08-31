use serde_json::{json, Map, Value};

use super::{types::quoted, UiBindingValue};

impl UiBindingValue {
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Unsigned(value) => Value::Number((*value).into()),
            Self::Signed(value) => Value::Number((*value).into()),
            Self::Float(value) => serde_json::Number::from_f64(*value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Self::Bool(value) => Value::Bool(*value),
            Self::Null => Value::Null,
            Self::Array(values) => Value::Array(values.iter().map(Self::to_json_value).collect()),
            Self::Record(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_json_value()))
                    .collect::<Map<_, _>>(),
            ),
            Self::Map(values) => json!({
                "$map": values
                    .iter()
                    .map(|(key, value)| json!({
                        "key": key.to_json_value(),
                        "value": value.to_json_value(),
                    }))
                    .collect::<Vec<_>>()
            }),
            Self::Enum(value) => {
                let mut projected = Map::new();
                projected.insert(
                    "type".to_string(),
                    Value::String(value.type_id().to_string()),
                );
                projected.insert(
                    "variant".to_string(),
                    Value::String(value.variant().to_string()),
                );
                if let Some(payload) = value.payload() {
                    projected.insert("payload".to_string(), payload.to_json_value());
                }
                json!({"$enum": projected})
            }
            Self::Asset(value) => json!({"$asset": value.locator()}),
            Self::Entity(value) => json!({
                "$entity": {
                    "id": value.entity_id(),
                    "generation": value.generation(),
                }
            }),
            Self::Optional(value) => value
                .as_deref()
                .map(Self::to_json_value)
                .unwrap_or(Value::Null),
            Self::CollectionView(value) => json!({
                "$collection_view": {
                    "provider_id": value.provider().id.as_str(),
                    "provider_version": value.provider().version.get(),
                    "item_schema_id": value.item_schema().id.as_str(),
                    "item_schema_version": value.item_schema().version.get(),
                    "revision": value.revision(),
                    "offset": value.offset(),
                    "length": value.length(),
                    "total_length": value.total_length(),
                }
            }),
        }
    }

    pub(crate) fn native_repr(&self) -> String {
        match self {
            Self::String(value) => quoted(value),
            Self::Unsigned(value) => value.to_string(),
            Self::Signed(value) => value.to_string(),
            Self::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                rendered
            }
            Self::Bool(value) => value.to_string(),
            Self::Null => "null".to_string(),
            Self::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::native_repr)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Record(fields) => format!(
                "record({})",
                fields
                    .iter()
                    .flat_map(|(field, value)| [quoted(field), value.native_repr()])
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Map(values) => format!(
                "map({})",
                values
                    .iter()
                    .flat_map(|(key, value)| [key.native_repr(), value.native_repr()])
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Enum(value) => {
                let mut arguments = vec![quoted(value.type_id()), quoted(value.variant())];
                if let Some(payload) = value.payload() {
                    arguments.push(payload.native_repr());
                }
                format!("enum({})", arguments.join(","))
            }
            Self::Asset(value) => format!("asset({})", quoted(value.locator())),
            Self::Entity(value) => {
                format!("entity({},{})", value.entity_id(), value.generation())
            }
            Self::Optional(value) => format!(
                "optional({})",
                value.as_deref().map(Self::native_repr).unwrap_or_default()
            ),
            Self::CollectionView(value) => format!(
                "collection_view({},{},{},{},{},{},{},{})",
                quoted(value.provider().id.as_str()),
                value.provider().version.get(),
                quoted(value.item_schema().id.as_str()),
                value.item_schema().version.get(),
                value.revision(),
                value.offset(),
                value.length(),
                value.total_length(),
            ),
        }
    }
}
