use zircon_runtime_interface::reflect::ReflectedValue;

use std::fmt;

const MAX_DECLARED_VALUE_TYPE_BYTES: usize = 256;
const MAX_DECLARED_VALUE_TYPE_DEPTH: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParsePolicy {
    General,
    StrictVm,
}

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
    DynamicList,
    DynamicMap,
    List(Box<Self>),
    Map(Box<Self>),
    Named(String),
}

impl DeclaredValueType {
    pub(super) fn parse(input: &str) -> Result<Self, String> {
        Self::parse_with_policy(input, ParsePolicy::General)
    }

    pub(super) fn parse_vm(input: &str) -> Result<Self, String> {
        Self::parse_with_policy(input, ParsePolicy::StrictVm)
    }

    fn parse_with_policy(input: &str, policy: ParsePolicy) -> Result<Self, String> {
        if input.is_empty() || input.trim() != input {
            return Err("declared value types must be non-empty and already trimmed".to_string());
        }
        if input.len() > MAX_DECLARED_VALUE_TYPE_BYTES {
            return Err(format!(
                "declared value types must not exceed {MAX_DECLARED_VALUE_TYPE_BYTES} bytes"
            ));
        }
        if !input.is_ascii() {
            return Err("declared value types must use ASCII wire text".to_string());
        }
        Self::parse_nested(input, policy, 0)
    }

    fn parse_nested(input: &str, policy: ParsePolicy, depth: usize) -> Result<Self, String> {
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

        if policy == ParsePolicy::General {
            let alias = match input {
                "bool" => Some(Self::Bool),
                "i8" | "i16" | "i32" | "i64" | "isize" => Some(Self::Integer),
                "u8" | "u16" | "u32" | "u64" | "usize" => Some(Self::Unsigned),
                "f32" | "f64" | "Real" => Some(Self::Scalar),
                "str" | "alloc::string::String" | "std::string::String" => Some(Self::String),
                "List" => Some(Self::DynamicList),
                "Map" => Some(Self::DynamicMap),
                _ => None,
            };
            if let Some(alias) = alias {
                return Ok(alias);
            }
        }

        if let Some(inner) = generic_inner(input, "List") {
            if inner.is_empty() {
                return Err("List<T> requires one declared item type".to_string());
            }
            let nested_depth = nested_depth(depth)?;
            return Ok(Self::List(Box::new(Self::parse_nested(
                inner,
                policy,
                nested_depth,
            )?)));
        }
        if let Some(inner) = generic_inner(input, "Map") {
            let (key, value) = split_map_arguments(inner)?;
            if key != "String" {
                return Err("Map<K, V> requires the JSON key type String".to_string());
            }
            let nested_depth = nested_depth(depth)?;
            return Ok(Self::Map(Box::new(Self::parse_nested(
                value,
                policy,
                nested_depth,
            )?)));
        }

        if policy == ParsePolicy::General && valid_named_type_path(input) {
            return Ok(Self::Named(input.to_string()));
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
            (Self::DynamicList, ReflectedValue::List(_))
            | (Self::DynamicMap, ReflectedValue::Map(_)) => true,
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

    pub(super) fn supports_numeric_metadata(&self) -> bool {
        matches!(self, Self::Integer | Self::Unsigned | Self::Scalar)
    }

    pub(super) fn supports_enum_metadata(&self) -> bool {
        matches!(self, Self::Enum)
    }

    pub(super) fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
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
            Self::DynamicList => formatter.write_str("List"),
            Self::DynamicMap => formatter.write_str("Map"),
            Self::List(item_type) => write!(formatter, "List<{item_type}>"),
            Self::Map(value_type) => write!(formatter, "Map<String, {value_type}>"),
            Self::Named(type_path) => formatter.write_str(type_path),
        }
    }
}

fn nested_depth(depth: usize) -> Result<usize, String> {
    let depth = depth
        .checked_add(1)
        .ok_or_else(|| "declared value type nesting depth overflowed".to_string())?;
    if depth > MAX_DECLARED_VALUE_TYPE_DEPTH {
        return Err(format!(
            "declared value type nesting must not exceed {MAX_DECLARED_VALUE_TYPE_DEPTH}"
        ));
    }
    Ok(depth)
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
            '<' => {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| "Map<K, V> nesting depth overflowed".to_string())?;
            }
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

fn valid_named_type_path(input: &str) -> bool {
    if input.contains('<') || input.contains('>') || input.contains(',') {
        return false;
    }
    if input.contains("::") {
        return !input.contains('.') && input.split("::").all(valid_identifier);
    }
    if input.contains('.') {
        let mut segments = input.split('.').peekable();
        while let Some(segment) = segments.next() {
            if segments.peek().is_none() {
                return valid_identifier(segment);
            }
            if !valid_namespace_key(segment) {
                return false;
            }
        }
        return false;
    }
    valid_identifier(input)
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn valid_namespace_key(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte == b'_' || byte == b'-' || byte.is_ascii_alphanumeric())
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}
