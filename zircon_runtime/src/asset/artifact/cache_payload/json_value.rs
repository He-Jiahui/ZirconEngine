use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

type ArtifactCacheJsonObject = BTreeMap<String, ArtifactCacheJsonValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheJsonNumber {
    I64(i64),
    U64(u64),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum ArtifactCacheJsonValue {
    Null,
    Bool(bool),
    Number(ArtifactCacheJsonNumber),
    String(String),
    Array(Vec<ArtifactCacheJsonValue>),
    Object(ArtifactCacheJsonObject),
}

impl ArtifactCacheJsonValue {
    pub(super) fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(*value),
            serde_json::Value::Number(value) => {
                if let Some(value) = value.as_i64() {
                    Self::Number(ArtifactCacheJsonNumber::I64(value))
                } else if let Some(value) = value.as_u64() {
                    Self::Number(ArtifactCacheJsonNumber::U64(value))
                } else {
                    Self::Number(ArtifactCacheJsonNumber::F64(value.as_f64().expect(
                        "serde_json::Number should expose finite f64 for floating payloads",
                    )))
                }
            }
            serde_json::Value::String(value) => Self::String(value.clone()),
            serde_json::Value::Array(values) => {
                Self::Array(values.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(values) => Self::Object(json_object_to_cache(values)),
        }
    }

    pub(super) fn into_json(self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(value) => serde_json::Value::Bool(value),
            Self::Number(value) => serde_json::Value::Number(match value {
                ArtifactCacheJsonNumber::I64(value) => serde_json::Number::from(value),
                ArtifactCacheJsonNumber::U64(value) => serde_json::Number::from(value),
                ArtifactCacheJsonNumber::F64(value) => {
                    serde_json::Number::from_f64(value).expect("cached JSON f64 should stay finite")
                }
            }),
            Self::String(value) => serde_json::Value::String(value),
            Self::Array(values) => serde_json::Value::Array(
                values
                    .into_iter()
                    .map(ArtifactCacheJsonValue::into_json)
                    .collect(),
            ),
            Self::Object(values) => serde_json::Value::Object(cache_object_to_json(values)),
        }
    }
}

pub(super) fn json_table_to_cache(
    table: &BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, ArtifactCacheJsonValue> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), ArtifactCacheJsonValue::from_json(value)))
        .collect()
}

pub(super) fn cache_table_to_json(
    table: BTreeMap<String, ArtifactCacheJsonValue>,
) -> BTreeMap<String, serde_json::Value> {
    table
        .into_iter()
        .map(|(key, value)| (key, value.into_json()))
        .collect()
}

fn json_object_to_cache(
    object: &serde_json::Map<String, serde_json::Value>,
) -> ArtifactCacheJsonObject {
    object
        .iter()
        .map(|(key, value)| (key.clone(), ArtifactCacheJsonValue::from_json(value)))
        .collect()
}

fn cache_object_to_json(
    object: ArtifactCacheJsonObject,
) -> serde_json::Map<String, serde_json::Value> {
    object
        .into_iter()
        .map(|(key, value)| (key, value.into_json()))
        .collect()
}
