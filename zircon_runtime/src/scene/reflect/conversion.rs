use crate::core::framework::scene::ScenePropertyValue;
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

pub fn reflected_from_scene_value(
    value: ScenePropertyValue,
) -> Result<ReflectedValue, ReflectError> {
    match value {
        ScenePropertyValue::Bool(value) => Ok(ReflectedValue::Bool(value)),
        ScenePropertyValue::Integer(value) => Ok(ReflectedValue::Integer(value)),
        ScenePropertyValue::Unsigned(value) => Ok(ReflectedValue::Unsigned(value)),
        ScenePropertyValue::Scalar(value) => Ok(ReflectedValue::Scalar(value)),
        ScenePropertyValue::Vec2(value) => Ok(ReflectedValue::Vec2(value)),
        ScenePropertyValue::Vec3(value) => Ok(ReflectedValue::Vec3(value)),
        ScenePropertyValue::Vec4(value) => Ok(ReflectedValue::Vec4(value)),
        ScenePropertyValue::Quaternion(value) => Ok(ReflectedValue::Quaternion(value)),
        ScenePropertyValue::String(value) => Ok(ReflectedValue::String(value)),
        ScenePropertyValue::Enum(value) => Ok(ReflectedValue::Enum(value)),
        ScenePropertyValue::Entity(value) => Ok(ReflectedValue::Entity(value)),
        ScenePropertyValue::Resource(value) => Ok(ReflectedValue::Resource(value)),
        ScenePropertyValue::AnimationParameter(_) => Err(ReflectError::UnsupportedConversion {
            source: "ScenePropertyValue::AnimationParameter".to_string(),
            target: "ReflectedValue".to_string(),
        }),
    }
}

pub fn scene_value_from_reflected(
    value: ReflectedValue,
) -> Result<ScenePropertyValue, ReflectError> {
    match value {
        ReflectedValue::Bool(value) => Ok(ScenePropertyValue::Bool(value)),
        ReflectedValue::Integer(value) => Ok(ScenePropertyValue::Integer(value)),
        ReflectedValue::Unsigned(value) => Ok(ScenePropertyValue::Unsigned(value)),
        ReflectedValue::Scalar(value) => {
            ensure_finite_scalar(value, "ReflectedValue::Scalar", "ScenePropertyValue")?;
            Ok(ScenePropertyValue::Scalar(value))
        }
        ReflectedValue::String(value) => Ok(ScenePropertyValue::String(value)),
        ReflectedValue::Enum(value) => Ok(ScenePropertyValue::Enum(value)),
        ReflectedValue::Vec2(value) => {
            ensure_finite_vector(&value, "ReflectedValue::Vec2", "ScenePropertyValue")?;
            Ok(ScenePropertyValue::Vec2(value))
        }
        ReflectedValue::Vec3(value) => {
            ensure_finite_vector(&value, "ReflectedValue::Vec3", "ScenePropertyValue")?;
            Ok(ScenePropertyValue::Vec3(value))
        }
        ReflectedValue::Vec4(value) => {
            ensure_finite_vector(&value, "ReflectedValue::Vec4", "ScenePropertyValue")?;
            Ok(ScenePropertyValue::Vec4(value))
        }
        ReflectedValue::Quaternion(value) => {
            ensure_finite_vector(&value, "ReflectedValue::Quaternion", "ScenePropertyValue")?;
            Ok(ScenePropertyValue::Quaternion(value))
        }
        ReflectedValue::Entity(value) => Ok(ScenePropertyValue::Entity(value)),
        ReflectedValue::Resource(value) => Ok(ScenePropertyValue::Resource(value)),
        ReflectedValue::Null => unsupported_reflected_to_scene("ReflectedValue::Null"),
        ReflectedValue::List(_) => unsupported_reflected_to_scene("ReflectedValue::List"),
        ReflectedValue::Map(_) => unsupported_reflected_to_scene("ReflectedValue::Map"),
        ReflectedValue::Json(_) => unsupported_reflected_to_scene("ReflectedValue::Json"),
    }
}

pub fn reflected_from_json(value: serde_json::Value) -> ReflectedValue {
    ReflectedValue::Json(value)
}

pub fn json_from_reflected(value: ReflectedValue) -> Result<serde_json::Value, ReflectError> {
    ensure_finite_reflected_value(&value, "serde_json::Value")?;
    serde_json::to_value(&value).map_err(|_| ReflectError::UnsupportedConversion {
        source: format!("ReflectedValue::{}", value.type_name()),
        target: "serde_json::Value".to_string(),
    })
}

fn unsupported_reflected_to_scene<T>(source: &str) -> Result<T, ReflectError> {
    Err(ReflectError::UnsupportedConversion {
        source: source.to_string(),
        target: "ScenePropertyValue".to_string(),
    })
}

fn ensure_finite_reflected_value(
    value: &ReflectedValue,
    target: &'static str,
) -> Result<(), ReflectError> {
    match value {
        ReflectedValue::Scalar(value) => {
            ensure_finite_scalar(*value, "ReflectedValue::Scalar", target)
        }
        ReflectedValue::Vec2(value) => ensure_finite_vector(value, "ReflectedValue::Vec2", target),
        ReflectedValue::Vec3(value) => ensure_finite_vector(value, "ReflectedValue::Vec3", target),
        ReflectedValue::Vec4(value) => ensure_finite_vector(value, "ReflectedValue::Vec4", target),
        ReflectedValue::Quaternion(value) => {
            ensure_finite_vector(value, "ReflectedValue::Quaternion", target)
        }
        ReflectedValue::List(values) => values
            .iter()
            .try_for_each(|value| ensure_finite_reflected_value(value, target)),
        ReflectedValue::Map(values) => values
            .values()
            .try_for_each(|value| ensure_finite_reflected_value(value, target)),
        ReflectedValue::Null
        | ReflectedValue::Bool(_)
        | ReflectedValue::Integer(_)
        | ReflectedValue::Unsigned(_)
        | ReflectedValue::String(_)
        | ReflectedValue::Enum(_)
        | ReflectedValue::Entity(_)
        | ReflectedValue::Resource(_)
        | ReflectedValue::Json(_) => Ok(()),
    }
}

fn ensure_finite_scalar(
    value: f32,
    source: &'static str,
    target: &'static str,
) -> Result<(), ReflectError> {
    value
        .is_finite()
        .then_some(())
        .ok_or_else(|| ReflectError::UnsupportedConversion {
            source: source.to_string(),
            target: target.to_string(),
        })
}

fn ensure_finite_vector(
    values: &[f32],
    source: &'static str,
    target: &'static str,
) -> Result<(), ReflectError> {
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(())
        .ok_or_else(|| ReflectError::UnsupportedConversion {
            source: source.to_string(),
            target: target.to_string(),
        })
}
