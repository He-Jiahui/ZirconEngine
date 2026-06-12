use std::str::FromStr;

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::framework::{animation::AnimationParameterValue, physics::PhysicsCombineRule};
use crate::core::math::{Quat, Real, Vec2, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, ResourceMarker};
use crate::scene::components::{JointKind, Mobility, RigidBodyType};
use crate::scene::EntityId;

pub(super) fn expect_segment_count(
    segments: &[String],
    expected: usize,
    property_path: &ComponentPropertyPath,
) -> Result<(), String> {
    if segments.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "property `{property_path}` expects {expected} segments, found {}",
            segments.len()
        ))
    }
}

pub(super) fn expect_segment(
    actual: &str,
    expected: &[&str],
    property_path: &ComponentPropertyPath,
) -> Result<(), String> {
    for candidate in expected {
        if *candidate == actual {
            return Ok(());
        }
    }

    Err(format!("unknown property `{property_path}`"))
}

pub(super) fn unknown_property_error(
    property_path: &ComponentPropertyPath,
) -> Result<bool, String> {
    Err(format!("unknown property `{property_path}`"))
}

pub(super) fn missing_component_error(
    entity: EntityId,
    property_path: &ComponentPropertyPath,
) -> Result<bool, String> {
    Err(format!(
        "entity {entity} does not expose property `{property_path}`"
    ))
}

pub(super) fn property_type_error<T>(
    property_path: &ComponentPropertyPath,
    expected: &str,
) -> Result<T, String> {
    Err(format!(
        "property `{property_path}` expected value of type {expected}"
    ))
}

pub(super) fn normalized_identifier(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        }
    }
    normalized
}

pub(super) fn normalized_identifier_matches(value: &str, target: &str) -> bool {
    let mut value_chars = value.chars();
    let mut target_chars = target.chars();

    loop {
        match (
            next_normalized_identifier_char(&mut value_chars),
            next_normalized_identifier_char(&mut target_chars),
        ) {
            (Some(value), Some(target)) if value == target => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn next_normalized_identifier_char(characters: &mut impl Iterator<Item = char>) -> Option<char> {
    for character in characters {
        if character.is_ascii_alphanumeric() {
            return Some(character.to_ascii_lowercase());
        }
    }

    None
}

pub(super) fn axis_index(
    axis: &str,
    property_path: &ComponentPropertyPath,
) -> Result<usize, String> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        _ => Err(format!("unknown axis in property `{property_path}`")),
    }
}

pub(super) fn quat_axis_index(
    axis: &str,
    property_path: &ComponentPropertyPath,
) -> Result<usize, String> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        "w" | "3" => Ok(3),
        _ => Err(format!(
            "unknown quaternion axis in property `{property_path}`"
        )),
    }
}

pub(super) fn expect_bool(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<bool, String> {
    let ScenePropertyValue::Bool(value) = value else {
        return Err(format!("property `{property_path}` expected bool"));
    };
    Ok(value)
}

pub(super) fn expect_string(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<String, String> {
    let ScenePropertyValue::String(value) = value else {
        return Err(format!("property `{property_path}` expected string"));
    };
    Ok(value)
}

pub(super) fn expect_enum(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<String, String> {
    let ScenePropertyValue::Enum(value) = value else {
        return Err(format!("property `{property_path}` expected enum string"));
    };
    Ok(value)
}

pub(super) fn expect_scalar(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<f32, String> {
    let value = match value {
        ScenePropertyValue::Scalar(value) => Ok(value),
        ScenePropertyValue::Integer(value) => Ok(value as f32),
        ScenePropertyValue::Unsigned(value) => Ok(value as f32),
        _ => Err(format!("property `{property_path}` expected scalar")),
    }?;
    validate_finite_scalar(value, property_path)?;
    Ok(value)
}

pub(super) fn expect_u32(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<u32, String> {
    match value {
        ScenePropertyValue::Unsigned(value) => Ok(value as u32),
        ScenePropertyValue::Integer(value) if value >= 0 => Ok(value as u32),
        _ => Err(format!(
            "property `{property_path}` expected unsigned integer"
        )),
    }
}

pub(super) fn expect_i32(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<i32, String> {
    match value {
        ScenePropertyValue::Integer(value) => match i32::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(format!("property `{property_path}` expected i32 integer")),
        },
        ScenePropertyValue::Unsigned(value) => match i32::try_from(value) {
            Ok(value) => Ok(value),
            Err(_) => Err(format!("property `{property_path}` expected i32 integer")),
        },
        _ => property_type_error(property_path, "integer"),
    }
}

pub(super) fn expect_vec3(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Vec3, String> {
    let ScenePropertyValue::Vec3(value) = value else {
        return Err(format!("property `{property_path}` expected vec3"));
    };
    validate_finite_array(&value, property_path, "vec3")?;
    Ok(Vec3::from_array(value))
}

pub(super) fn expect_vec2(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Vec2, String> {
    let ScenePropertyValue::Vec2(value) = value else {
        return Err(format!("property `{property_path}` expected vec2"));
    };
    validate_finite_array(&value, property_path, "vec2")?;
    Ok(Vec2::from_array(value))
}

pub(super) fn expect_vec4(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Vec4, String> {
    let ScenePropertyValue::Vec4(value) = value else {
        return Err(format!("property `{property_path}` expected vec4"));
    };
    validate_finite_array(&value, property_path, "vec4")?;
    Ok(Vec4::from_array(value))
}

pub(super) fn expect_quat(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Quat, String> {
    let ScenePropertyValue::Quaternion(value) = value else {
        return Err(format!("property `{property_path}` expected quaternion"));
    };
    validate_quat_array(value, property_path)?;
    Ok(Quat::from_array(value))
}

pub(super) fn validate_quat_array(
    value: [Real; 4],
    property_path: &ComponentPropertyPath,
) -> Result<(), String> {
    validate_finite_array(&value, property_path, "quaternion")?;
    let mut length_squared = 0.0;
    for component in value {
        length_squared += component * component;
    }

    if length_squared <= Real::EPSILON {
        return Err(format!(
            "property `{property_path}` rejects zero-length quaternion"
        ));
    }
    Ok(())
}

fn validate_finite_scalar(
    value: Real,
    property_path: &ComponentPropertyPath,
) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!("property `{property_path}` expected finite scalar"))
    }
}

fn validate_finite_array(
    value: &[Real],
    property_path: &ComponentPropertyPath,
    expected: &str,
) -> Result<(), String> {
    for component in value {
        if !component.is_finite() {
            return Err(format!(
                "property `{property_path}` expected finite {expected}"
            ));
        }
    }

    Ok(())
}

pub(super) fn expect_resource_id(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<ResourceId, String> {
    let ScenePropertyValue::Resource(value) = value else {
        return Err(format!("property `{property_path}` expected resource id"));
    };
    match ResourceId::from_str(&value) {
        Ok(resource_id) => Ok(resource_id),
        Err(error) => Err(format!(
            "property `{property_path}` has invalid resource id: {error}"
        )),
    }
}

pub(super) fn expect_animation_parameter(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<AnimationParameterValue, String> {
    let ScenePropertyValue::AnimationParameter(value) = value else {
        return Err(format!(
            "property `{property_path}` expected animation parameter"
        ));
    };
    Ok(value)
}

pub(super) fn parse_mobility(value: &str) -> Result<Mobility, String> {
    if normalized_identifier_matches(value, "dynamic") {
        Ok(Mobility::Dynamic)
    } else if normalized_identifier_matches(value, "static") {
        Ok(Mobility::Static)
    } else {
        Err(format!("unsupported mobility `{value}`"))
    }
}

pub(super) fn parse_rigid_body_type(value: &str) -> Result<RigidBodyType, String> {
    if normalized_identifier_matches(value, "dynamic") {
        Ok(RigidBodyType::Dynamic)
    } else if normalized_identifier_matches(value, "static") {
        Ok(RigidBodyType::Static)
    } else if normalized_identifier_matches(value, "kinematic") {
        Ok(RigidBodyType::Kinematic)
    } else {
        Err(format!("unsupported rigid body type `{value}`"))
    }
}

pub(super) fn parse_joint_kind(value: &str) -> Result<JointKind, String> {
    if normalized_identifier_matches(value, "fixed") {
        Ok(JointKind::Fixed)
    } else if normalized_identifier_matches(value, "distance") {
        Ok(JointKind::Distance)
    } else if normalized_identifier_matches(value, "hinge") {
        Ok(JointKind::Hinge)
    } else if normalized_identifier_matches(value, "slider") {
        Ok(JointKind::Slider)
    } else if normalized_identifier_matches(value, "conetwist") {
        Ok(JointKind::ConeTwist)
    } else if normalized_identifier_matches(value, "generic6dof")
        || normalized_identifier_matches(value, "d6")
        || normalized_identifier_matches(value, "sixdof")
    {
        Ok(JointKind::Generic6Dof)
    } else {
        Err(format!("unsupported joint kind `{value}`"))
    }
}

pub(super) fn parse_combine_rule(value: &str) -> Result<PhysicsCombineRule, String> {
    if normalized_identifier_matches(value, "average") {
        Ok(PhysicsCombineRule::Average)
    } else if normalized_identifier_matches(value, "minimum") {
        Ok(PhysicsCombineRule::Minimum)
    } else if normalized_identifier_matches(value, "maximum") {
        Ok(PhysicsCombineRule::Maximum)
    } else if normalized_identifier_matches(value, "multiply") {
        Ok(PhysicsCombineRule::Multiply)
    } else {
        Err(format!("unsupported combine rule `{value}`"))
    }
}

pub(super) fn combine_rule_label(rule: PhysicsCombineRule) -> &'static str {
    match rule {
        PhysicsCombineRule::Average => "average",
        PhysicsCombineRule::Minimum => "minimum",
        PhysicsCombineRule::Maximum => "maximum",
        PhysicsCombineRule::Multiply => "multiply",
    }
}

pub(super) fn set_animation_player_like_property<TMarker>(
    segments: &[String],
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
    handle: &mut ResourceHandle<TMarker>,
    playback_speed: &mut f32,
    time_seconds: &mut f32,
    weight: Option<&mut f32>,
    looping: &mut bool,
    playing: &mut bool,
) -> Result<bool, String>
where
    TMarker: ResourceMarker,
{
    match segments {
        [field] if field == "clip" || field == "sequence" => {
            let next = expect_resource_id(value, property_path)?;
            if handle.id() == next {
                Ok(false)
            } else {
                *handle = ResourceHandle::new(next);
                Ok(true)
            }
        }
        [field] if field == "playbackspeed" => {
            let next = expect_scalar(value, property_path)?;
            if *playback_speed == next {
                Ok(false)
            } else {
                *playback_speed = next;
                Ok(true)
            }
        }
        [field] if field == "timeseconds" => {
            let next = expect_scalar(value, property_path)?;
            if *time_seconds == next {
                Ok(false)
            } else {
                *time_seconds = next;
                Ok(true)
            }
        }
        [field] if field == "weight" => {
            let Some(weight) = weight else {
                return unknown_property_error(property_path);
            };
            let next = expect_scalar(value, property_path)?;
            if *weight == next {
                Ok(false)
            } else {
                *weight = next;
                Ok(true)
            }
        }
        [field] if field == "looping" => {
            let next = expect_bool(value, property_path)?;
            if *looping == next {
                Ok(false)
            } else {
                *looping = next;
                Ok(true)
            }
        }
        [field] if field == "playing" => {
            let next = expect_bool(value, property_path)?;
            if *playing == next {
                Ok(false)
            } else {
                *playing = next;
                Ok(true)
            }
        }
        _ => unknown_property_error(property_path),
    }
}
