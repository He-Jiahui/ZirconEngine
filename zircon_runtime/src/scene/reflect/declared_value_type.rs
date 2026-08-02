use zircon_runtime_interface::reflect::ReflectedValue;

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DeclaredValueType {
    Null,
    Bool,
    Integer,
    Unsigned,
    Scalar,
    String,
    Enum,
    Vec2,
    Vec3,
    Vec4,
    Quaternion,
    Entity,
    Resource,
    Json,
    List(Box<Self>),
    Map(Box<Self>),
}

impl DeclaredValueType {
    pub(super) fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() || input.trim() != input {
            return Err("declared value types must be non-empty and already trimmed".to_string());
        }
        let primitive = match input {
            "Null" => Some(Self::Null),
            "Bool" => Some(Self::Bool),
            "Integer" => Some(Self::Integer),
            "Unsigned" => Some(Self::Unsigned),
            "Scalar" => Some(Self::Scalar),
            "String" => Some(Self::String),
            "Enum" => Some(Self::Enum),
            "Vec2" => Some(Self::Vec2),
            "Vec3" => Some(Self::Vec3),
            "Vec4" => Some(Self::Vec4),
            "Quaternion" => Some(Self::Quaternion),
            "Entity" => Some(Self::Entity),
            "Resource" => Some(Self::Resource),
            "Json" => Some(Self::Json),
            _ => None,
        };
        if let Some(primitive) = primitive {
            return Ok(primitive);
        }

        if let Some(inner) = generic_inner(input, "List") {
            if inner.is_empty() {
                return Err("List<T> requires one declared item type".to_string());
            }
            return Ok(Self::List(Box::new(Self::parse(inner)?)));
        }
        if let Some(inner) = generic_inner(input, "Map") {
            let (key, value) = split_map_arguments(inner)?;
            if key != "String" {
                return Err("Map<K, V> requires the JSON key type String".to_string());
            }
            return Ok(Self::Map(Box::new(Self::parse(value)?)));
        }

        Err(format!("unsupported declared value type `{input}`"))
    }

    pub(super) fn matches_reflected(&self, value: &ReflectedValue) -> bool {
        match (self, value) {
            (Self::Null, ReflectedValue::Null)
            | (Self::Bool, ReflectedValue::Bool(_))
            | (Self::Integer, ReflectedValue::Integer(_))
            | (Self::Unsigned, ReflectedValue::Unsigned(_))
            | (Self::String, ReflectedValue::String(_))
            | (Self::Enum, ReflectedValue::Enum(_))
            | (Self::Entity, ReflectedValue::Entity(_))
            | (Self::Entity, ReflectedValue::Null)
            | (Self::Resource, ReflectedValue::Resource(_))
            | (Self::Json, ReflectedValue::Json(_)) => true,
            (Self::Scalar, ReflectedValue::Scalar(value)) => value.is_finite(),
            (Self::Vec2, ReflectedValue::Vec2(values)) => finite(values),
            (Self::Vec3, ReflectedValue::Vec3(values)) => finite(values),
            (Self::Vec4, ReflectedValue::Vec4(values))
            | (Self::Quaternion, ReflectedValue::Quaternion(values)) => finite(values),
            (Self::List(item_type), ReflectedValue::List(values)) => values
                .iter()
                .all(|value| item_type.matches_reflected(value)),
            (Self::Map(value_type), ReflectedValue::Map(values)) => values
                .values()
                .all(|value| value_type.matches_reflected(value)),
            _ => false,
        }
    }
}

impl fmt::Display for DeclaredValueType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("Null"),
            Self::Bool => formatter.write_str("Bool"),
            Self::Integer => formatter.write_str("Integer"),
            Self::Unsigned => formatter.write_str("Unsigned"),
            Self::Scalar => formatter.write_str("Scalar"),
            Self::String => formatter.write_str("String"),
            Self::Enum => formatter.write_str("Enum"),
            Self::Vec2 => formatter.write_str("Vec2"),
            Self::Vec3 => formatter.write_str("Vec3"),
            Self::Vec4 => formatter.write_str("Vec4"),
            Self::Quaternion => formatter.write_str("Quaternion"),
            Self::Entity => formatter.write_str("Entity"),
            Self::Resource => formatter.write_str("Resource"),
            Self::Json => formatter.write_str("Json"),
            Self::List(item_type) => write!(formatter, "List<{item_type}>"),
            Self::Map(value_type) => write!(formatter, "Map<String, {value_type}>"),
        }
    }
}

fn generic_inner<'a>(input: &'a str, owner: &str) -> Option<&'a str> {
    input
        .strip_prefix(owner)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

fn split_map_arguments(inner: &str) -> Result<(&str, &str), String> {
    let mut depth = 0_usize;
    let mut separator = None;
    for (index, character) in inner.char_indices() {
        match character {
            '<' => depth = depth.saturating_add(1),
            '>' if depth == 0 => {
                return Err("Map<K, V> contains an unmatched closing bracket".to_string());
            }
            '>' => depth -= 1,
            ',' if depth == 0 => {
                if separator.replace(index).is_some() {
                    return Err("Map<K, V> requires exactly two arguments".to_string());
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("Map<K, V> contains an unclosed nested type".to_string());
    }
    let Some(separator) = separator else {
        return Err("Map<K, V> requires String key and value arguments".to_string());
    };
    let key = inner[..separator].trim();
    let value = inner[separator + 1..].trim();
    if key.is_empty() || value.is_empty() {
        return Err("Map<K, V> arguments must not be empty".to_string());
    }
    Ok((key, value))
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}
