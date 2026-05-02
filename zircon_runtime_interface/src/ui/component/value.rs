use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::{map::Map, Value as TomlValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Describes the typed value family accepted by Runtime UI component props and state.
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
/// Stores a typed Runtime UI value for authored props, retained state, and event payloads.
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
    /// Returns the value kind represented by this concrete typed value.
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

    /// Converts numeric-like values into `f64` for numeric component reducers.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::String(value) => value.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Formats a compact user-facing value summary for generic host projection.
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

    /// Converts a TOML value into the nearest Runtime UI typed value.
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

    /// Converts a TOML value into a typed Runtime UI value required by a component schema.
    pub fn from_toml_with_kind(value: &TomlValue, kind: UiValueKind) -> Option<Self> {
        match kind {
            UiValueKind::Any => Some(Self::from_toml(value)),
            UiValueKind::Bool => value.as_bool().map(Self::Bool),
            UiValueKind::Int => value.as_integer().map(Self::Int),
            UiValueKind::Float => match value {
                TomlValue::Integer(value) => Some(Self::Float(*value as f64)),
                TomlValue::Float(value) => Some(Self::Float(*value)),
                _ => None,
            },
            UiValueKind::String => value.as_str().map(|value| Self::String(value.to_string())),
            UiValueKind::Color => value.as_str().map(|value| Self::Color(value.to_string())),
            UiValueKind::Vec2 => fixed_float_array::<2>(value).map(Self::Vec2),
            UiValueKind::Vec3 => fixed_float_array::<3>(value).map(Self::Vec3),
            UiValueKind::Vec4 => fixed_float_array::<4>(value).map(Self::Vec4),
            UiValueKind::AssetRef => value
                .as_str()
                .map(|value| Self::AssetRef(value.to_string())),
            UiValueKind::InstanceRef => value
                .as_str()
                .map(|value| Self::InstanceRef(value.to_string())),
            UiValueKind::Array => value
                .as_array()
                .map(|values| Self::Array(values.iter().map(Self::from_toml).collect::<Vec<_>>())),
            UiValueKind::Map => value.as_table().map(|values| {
                Self::Map(
                    values
                        .iter()
                        .map(|(key, value)| (key.clone(), Self::from_toml(value)))
                        .collect(),
                )
            }),
            UiValueKind::Enum => value.as_str().map(|value| Self::Enum(value.to_string())),
            UiValueKind::Flags => value.as_array().and_then(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().map(str::to_string))
                    .collect::<Option<Vec<_>>>()
                    .map(Self::Flags)
            }),
            UiValueKind::Null => None,
        }
    }

    /// Converts the Runtime UI typed value back into TOML for compiled template attributes.
    pub fn to_toml(&self) -> TomlValue {
        match self {
            Self::Bool(value) => TomlValue::Boolean(*value),
            Self::Int(value) => TomlValue::Integer(*value),
            Self::Float(value) => TomlValue::Float(*value),
            Self::String(value)
            | Self::Color(value)
            | Self::AssetRef(value)
            | Self::InstanceRef(value)
            | Self::Enum(value) => TomlValue::String(value.clone()),
            Self::Vec2(value) => float_array_to_toml(value),
            Self::Vec3(value) => float_array_to_toml(value),
            Self::Vec4(value) => float_array_to_toml(value),
            Self::Array(values) => TomlValue::Array(values.iter().map(Self::to_toml).collect()),
            Self::Map(values) => TomlValue::Table(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_toml()))
                    .collect::<Map<String, TomlValue>>(),
            ),
            Self::Flags(values) => TomlValue::Array(
                values
                    .iter()
                    .map(|value| TomlValue::String(value.clone()))
                    .collect(),
            ),
            Self::Null => TomlValue::String(String::new()),
        }
    }
}

fn fixed_float_array<const N: usize>(value: &TomlValue) -> Option<[f64; N]> {
    let values = value.as_array()?;
    if values.len() != N {
        return None;
    }

    let floats = values
        .iter()
        .map(|value| match value {
            TomlValue::Integer(value) => Some(*value as f64),
            TomlValue::Float(value) => Some(*value),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    floats.try_into().ok()
}

fn float_array_to_toml<const N: usize>(values: &[f64; N]) -> TomlValue {
    TomlValue::Array(values.iter().copied().map(TomlValue::Float).collect())
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
