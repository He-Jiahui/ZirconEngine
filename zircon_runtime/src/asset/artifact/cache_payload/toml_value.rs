use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(super) type ArtifactCacheTomlTable = BTreeMap<String, ArtifactCacheTomlValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum ArtifactCacheTomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Array(Vec<ArtifactCacheTomlValue>),
    Table(ArtifactCacheTomlTable),
}

impl ArtifactCacheTomlValue {
    pub(super) fn from_toml(value: &toml::Value) -> Self {
        match value {
            toml::Value::String(value) => Self::String(value.clone()),
            toml::Value::Integer(value) => Self::Integer(*value),
            toml::Value::Float(value) => Self::Float(*value),
            toml::Value::Boolean(value) => Self::Boolean(*value),
            toml::Value::Datetime(value) => Self::Datetime(value.to_string()),
            toml::Value::Array(values) => Self::Array(values.iter().map(Self::from_toml).collect()),
            toml::Value::Table(table) => Self::Table(toml_table_to_cache(table)),
        }
    }

    pub(super) fn into_toml(self) -> Result<toml::Value, String> {
        Ok(match self {
            Self::String(value) => toml::Value::String(value),
            Self::Integer(value) => toml::Value::Integer(value),
            Self::Float(value) => toml::Value::Float(value),
            Self::Boolean(value) => toml::Value::Boolean(value),
            Self::Datetime(value) => toml::Value::Datetime(
                value
                    .parse::<toml::value::Datetime>()
                    .map_err(|error| format!("invalid cached TOML datetime `{value}`: {error}"))?,
            ),
            Self::Array(values) => toml::Value::Array(
                values
                    .into_iter()
                    .map(Self::into_toml)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Self::Table(table) => toml::Value::Table(cache_table_to_toml(table)?),
        })
    }
}

pub(super) fn toml_table_like_to_cache(
    table: &BTreeMap<String, toml::Value>,
) -> BTreeMap<String, ArtifactCacheTomlValue> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), ArtifactCacheTomlValue::from_toml(value)))
        .collect()
}

pub(super) fn cache_table_like_to_toml(
    table: BTreeMap<String, ArtifactCacheTomlValue>,
) -> Result<BTreeMap<String, toml::Value>, String> {
    table
        .into_iter()
        .map(|(key, value)| value.into_toml().map(|value| (key, value)))
        .collect()
}

pub(super) fn toml_table_to_cache(table: &toml::Table) -> ArtifactCacheTomlTable {
    table
        .iter()
        .map(|(key, value)| (key.clone(), ArtifactCacheTomlValue::from_toml(value)))
        .collect()
}

pub(super) fn cache_table_to_toml(table: ArtifactCacheTomlTable) -> Result<toml::Table, String> {
    let mut output = toml::Table::new();
    for (key, value) in table {
        output.insert(key, value.into_toml()?);
    }
    Ok(output)
}
