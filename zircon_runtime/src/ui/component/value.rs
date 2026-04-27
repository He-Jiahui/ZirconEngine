use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiValueKind {
    Any,
    Bool,
    Int,
    Float,
    String,
    Color,
    Vec2,
    Vec3,
    Vec4,
    AssetRef,
    InstanceRef,
    Array,
    Map,
    Enum,
    Flags,
    Null,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(String),
    Vec2([f64; 2]),
    Vec3([f64; 3]),
    Vec4([f64; 4]),
    AssetRef(String),
    InstanceRef(String),
    Array(Vec<UiValue>),
    Map(BTreeMap<String, UiValue>),
    Enum(String),
    Flags(Vec<String>),
    Null,
}

impl UiValue {
    pub fn kind(&self) -> UiValueKind {
        match self {
            Self::Bool(_) => UiValueKind::Bool,
            Self::Int(_) => UiValueKind::Int,
            Self::Float(_) => UiValueKind::Float,
            Self::String(_) => UiValueKind::String,
            Self::Color(_) => UiValueKind::Color,
            Self::Vec2(_) => UiValueKind::Vec2,
            Self::Vec3(_) => UiValueKind::Vec3,
            Self::Vec4(_) => UiValueKind::Vec4,
            Self::AssetRef(_) => UiValueKind::AssetRef,
            Self::InstanceRef(_) => UiValueKind::InstanceRef,
            Self::Array(_) => UiValueKind::Array,
            Self::Map(_) => UiValueKind::Map,
            Self::Enum(_) => UiValueKind::Enum,
            Self::Flags(_) => UiValueKind::Flags,
            Self::Null => UiValueKind::Null,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::String(value) => value.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => trim_float(*value),
            Self::String(value)
            | Self::Color(value)
            | Self::AssetRef(value)
            | Self::InstanceRef(value)
            | Self::Enum(value) => value.clone(),
            Self::Vec2(value) => format!("{}, {}", trim_float(value[0]), trim_float(value[1])),
            Self::Vec3(value) => format!(
                "{}, {}, {}",
                trim_float(value[0]),
                trim_float(value[1]),
                trim_float(value[2])
            ),
            Self::Vec4(value) => format!(
                "{}, {}, {}, {}",
                trim_float(value[0]),
                trim_float(value[1]),
                trim_float(value[2]),
                trim_float(value[3])
            ),
            Self::Array(values) => format!("{} items", values.len()),
            Self::Map(values) => format!("{} entries", values.len()),
            Self::Flags(values) => values.join(", "),
            Self::Null => String::new(),
        }
    }

    pub fn from_toml(value: &TomlValue) -> Self {
        match value {
            TomlValue::String(value) => Self::String(value.clone()),
            TomlValue::Integer(value) => Self::Int(*value),
            TomlValue::Float(value) => Self::Float(*value),
            TomlValue::Boolean(value) => Self::Bool(*value),
            TomlValue::Array(values) => Self::Array(values.iter().map(Self::from_toml).collect()),
            TomlValue::Table(values) => Self::Map(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::from_toml(value)))
                    .collect(),
            ),
            TomlValue::Datetime(value) => Self::String(value.to_string()),
        }
    }
}

fn trim_float(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < f64::EPSILON {
        format!("{rounded:.0}")
    } else {
        let mut text = format!("{value:.3}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}
