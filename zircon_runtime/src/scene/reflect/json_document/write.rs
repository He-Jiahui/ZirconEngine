use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::{write_versioned_text, VersionedSchema, WriteError};

use super::document::ReflectedJsonDocument;
use super::ReflectedJsonError;

/// Writes the current reflected-JSON schema with canonical text formatting.
pub fn json_from_reflected(value: &ReflectedValue) -> Result<String, ReflectedJsonError> {
    if !reflected_value_is_finite(value) {
        return Err(WriteError::PayloadEncode {
            schema_id: ReflectedJsonDocument::SCHEMA.as_str().to_string(),
            schema_version: ReflectedJsonDocument::VERSION,
            source: serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "reflected value contains a non-finite scalar",
            )),
        }
        .into());
    }
    Ok(write_versioned_text(&ReflectedJsonDocument {
        value: value.clone(),
    })?)
}

fn reflected_value_is_finite(value: &ReflectedValue) -> bool {
    match value {
        ReflectedValue::Scalar(value) => value.is_finite(),
        ReflectedValue::Vec2(values) => values.iter().all(|value| value.is_finite()),
        ReflectedValue::Vec3(values) => values.iter().all(|value| value.is_finite()),
        ReflectedValue::Vec4(values) | ReflectedValue::Quaternion(values) => {
            values.iter().all(|value| value.is_finite())
        }
        ReflectedValue::List(values) => values.iter().all(reflected_value_is_finite),
        ReflectedValue::Map(values) => values.values().all(reflected_value_is_finite),
        ReflectedValue::Null
        | ReflectedValue::Bool(_)
        | ReflectedValue::Integer(_)
        | ReflectedValue::Unsigned(_)
        | ReflectedValue::String(_)
        | ReflectedValue::Enum(_)
        | ReflectedValue::Entity(_)
        | ReflectedValue::Resource(_)
        | ReflectedValue::Json(_) => true,
    }
}
