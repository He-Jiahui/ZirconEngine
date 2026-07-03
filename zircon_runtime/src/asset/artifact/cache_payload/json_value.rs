use std::collections::BTreeMap;

use crate::asset::AssetImportError;
use serde::{Deserialize, Serialize};

type ArtifactCacheJsonObject = BTreeMap<String, ArtifactCacheJsonValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheJsonNumber {
    I64(i64),
    U64(u64),
    F64(f64),
    Decimal(String),
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
                } else if let Some(value) = value.as_f64().filter(|value| value.is_finite()) {
                    Self::Number(ArtifactCacheJsonNumber::F64(value))
                } else {
                    Self::Number(ArtifactCacheJsonNumber::Decimal(value.to_string()))
                }
            }
            serde_json::Value::String(value) => Self::String(value.clone()),
            serde_json::Value::Array(values) => {
                Self::Array(values.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(values) => Self::Object(json_object_to_cache(values)),
        }
    }

    pub(super) fn into_json(self) -> Result<serde_json::Value, AssetImportError> {
        match self {
            Self::Null => Ok(serde_json::Value::Null),
            Self::Bool(value) => Ok(serde_json::Value::Bool(value)),
            Self::Number(value) => Ok(serde_json::Value::Number(value.into_json_number()?)),
            Self::String(value) => Ok(serde_json::Value::String(value)),
            Self::Array(values) => Ok(serde_json::Value::Array(
                values
                    .into_iter()
                    .map(ArtifactCacheJsonValue::into_json)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Self::Object(values) => Ok(serde_json::Value::Object(cache_object_to_json(values)?)),
        }
    }
}

impl ArtifactCacheJsonNumber {
    fn into_json_number(self) -> Result<serde_json::Number, AssetImportError> {
        match self {
            Self::I64(value) => Ok(serde_json::Number::from(value)),
            Self::U64(value) => Ok(serde_json::Number::from(value)),
            Self::F64(value) => serde_json::Number::from_f64(value).ok_or_else(|| {
                AssetImportError::CachedJsonNonFiniteNumber {
                    value: value.to_string(),
                }
            }),
            Self::Decimal(value) => match serde_json::from_str::<serde_json::Value>(&value)
                .map_err(|source| AssetImportError::CachedJsonNumberParse {
                    value: value.clone(),
                    source,
                })? {
                serde_json::Value::Number(number) => Ok(number),
                _ => Err(AssetImportError::CachedJsonNonFiniteNumber { value }),
            },
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
) -> Result<BTreeMap<String, serde_json::Value>, AssetImportError> {
    table
        .into_iter()
        .map(|(key, value)| value.into_json().map(|value| (key, value)))
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
) -> Result<serde_json::Map<String, serde_json::Value>, AssetImportError> {
    let mut output = serde_json::Map::new();
    for (key, value) in object {
        output.insert(key, value.into_json()?);
    }
    Ok(output)
}
