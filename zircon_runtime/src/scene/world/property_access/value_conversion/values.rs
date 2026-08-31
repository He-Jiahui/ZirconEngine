use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Quat, Real, Vec2, Vec3, Vec4};
use crate::scene::{SceneError, SceneResult};

use super::errors::property_type_error;

pub(in crate::scene::world::property_access) fn expect_bool(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    let ScenePropertyValue::Bool(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "bool".to_string(),
        });
    };
    Ok(value)
}

pub(in crate::scene::world::property_access) fn expect_string(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<String> {
    let ScenePropertyValue::String(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "string".to_string(),
        });
    };
    Ok(value)
}

pub(in crate::scene::world::property_access) fn expect_enum(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<String> {
    let ScenePropertyValue::Enum(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "enum string".to_string(),
        });
    };
    Ok(value)
}

pub(in crate::scene::world::property_access) fn expect_scalar(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<f32> {
    let value = match value {
        ScenePropertyValue::Scalar(value) => Ok(value),
        ScenePropertyValue::Integer(value) => Ok(value as f32),
        ScenePropertyValue::Unsigned(value) => Ok(value as f32),
        _ => Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "scalar".to_string(),
        }),
    }?;
    validate_finite_scalar(value, property_path)?;
    Ok(value)
}

pub(in crate::scene::world::property_access) fn expect_u32(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<u32> {
    match value {
        ScenePropertyValue::Unsigned(value) => Ok(value as u32),
        ScenePropertyValue::Integer(value) if value >= 0 => Ok(value as u32),
        _ => Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "unsigned integer".to_string(),
        }),
    }
}

pub(in crate::scene::world::property_access) fn expect_i32(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<i32> {
    match value {
        ScenePropertyValue::Integer(value) => match i32::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(SceneError::PropertyTypeMismatch {
                property_path: property_path.to_string(),
                expected: "i32 integer".to_string(),
            }),
        },
        ScenePropertyValue::Unsigned(value) => match i32::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(SceneError::PropertyTypeMismatch {
                property_path: property_path.to_string(),
                expected: "i32 integer".to_string(),
            }),
        },
        _ => property_type_error(property_path, "integer"),
    }
}

pub(in crate::scene::world::property_access) fn expect_vec3(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<Vec3> {
    let ScenePropertyValue::Vec3(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "vec3".to_string(),
        });
    };
    validate_finite_array(&value, property_path, "vec3")?;
    Ok(Vec3::from_array(value))
}

pub(in crate::scene::world::property_access) fn expect_vec2(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<Vec2> {
    let ScenePropertyValue::Vec2(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "vec2".to_string(),
        });
    };
    validate_finite_array(&value, property_path, "vec2")?;
    Ok(Vec2::from_array(value))
}

pub(in crate::scene::world::property_access) fn expect_vec4(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<Vec4> {
    let ScenePropertyValue::Vec4(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "vec4".to_string(),
        });
    };
    validate_finite_array(&value, property_path, "vec4")?;
    Ok(Vec4::from_array(value))
}

pub(in crate::scene::world::property_access) fn expect_quat(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<Quat> {
    let ScenePropertyValue::Quaternion(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "quaternion".to_string(),
        });
    };
    validate_quat_array(value, property_path)?;
    Ok(Quat::from_array(value))
}

pub(in crate::scene::world::property_access) fn validate_quat_array(
    value: [Real; 4],
    property_path: &ComponentPropertyPath,
) -> SceneResult<()> {
    validate_finite_array(&value, property_path, "quaternion")?;
    let mut length_squared = 0.0;
    for component in value {
        length_squared += component * component;
    }

    if length_squared <= Real::EPSILON {
        return Err(SceneError::ZeroLengthQuaternion {
            property_path: property_path.to_string(),
        });
    }
    Ok(())
}

fn validate_finite_scalar(value: Real, property_path: &ComponentPropertyPath) -> SceneResult<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SceneError::NonFinitePropertyValue {
            property_path: property_path.to_string(),
            expected: "scalar",
        })
    }
}

fn validate_finite_array(
    value: &[Real],
    property_path: &ComponentPropertyPath,
    expected: &'static str,
) -> SceneResult<()> {
    for component in value {
        if !component.is_finite() {
            return Err(SceneError::NonFinitePropertyValue {
                property_path: property_path.to_string(),
                expected,
            });
        }
    }

    Ok(())
}
