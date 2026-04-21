use std::str::FromStr;

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::framework::{animation::AnimationParameterValue, physics::PhysicsCombineRule};
use crate::core::math::{Quat, Vec3, Vec4};
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
    if expected.iter().any(|candidate| *candidate == actual) {
        Ok(())
    } else {
        Err(format!("unknown property `{property_path}`"))
    }
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

pub(super) fn property_type_error(
    property_path: &ComponentPropertyPath,
    expected: &str,
) -> Result<bool, String> {
    Err(format!(
        "property `{property_path}` expected value of type {expected}"
    ))
}

pub(super) fn normalized_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
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
    match value {
        ScenePropertyValue::Scalar(value) => Ok(value),
        ScenePropertyValue::Integer(value) => Ok(value as f32),
        ScenePropertyValue::Unsigned(value) => Ok(value as f32),
        _ => Err(format!("property `{property_path}` expected scalar")),
    }
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

pub(super) fn expect_vec3(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Vec3, String> {
    let ScenePropertyValue::Vec3(value) = value else {
        return Err(format!("property `{property_path}` expected vec3"));
    };
    Ok(Vec3::from_array(value))
}

pub(super) fn expect_vec4(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Vec4, String> {
    let ScenePropertyValue::Vec4(value) = value else {
        return Err(format!("property `{property_path}` expected vec4"));
    };
    Ok(Vec4::from_array(value))
}

pub(super) fn expect_quat(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<Quat, String> {
    let ScenePropertyValue::Quaternion(value) = value else {
        return Err(format!("property `{property_path}` expected quaternion"));
    };
    Ok(Quat::from_array(value))
}

pub(super) fn expect_resource_id(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> Result<ResourceId, String> {
    let ScenePropertyValue::Resource(value) = value else {
        return Err(format!("property `{property_path}` expected resource id"));
    };
    ResourceId::from_str(&value)
        .map_err(|error| format!("property `{property_path}` has invalid resource id: {error}"))
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
    match normalized_identifier(value).as_str() {
        "dynamic" => Ok(Mobility::Dynamic),
        "static" => Ok(Mobility::Static),
        _ => Err(format!("unsupported mobility `{value}`")),
    }
}

pub(super) fn parse_rigid_body_type(value: &str) -> Result<RigidBodyType, String> {
    match normalized_identifier(value).as_str() {
        "dynamic" => Ok(RigidBodyType::Dynamic),
        "static" => Ok(RigidBodyType::Static),
        "kinematic" => Ok(RigidBodyType::Kinematic),
        _ => Err(format!("unsupported rigid body type `{value}`")),
    }
}

pub(super) fn parse_joint_kind(value: &str) -> Result<JointKind, String> {
    match normalized_identifier(value).as_str() {
        "fixed" => Ok(JointKind::Fixed),
        "distance" => Ok(JointKind::Distance),
        "hinge" => Ok(JointKind::Hinge),
        _ => Err(format!("unsupported joint kind `{value}`")),
    }
}

pub(super) fn parse_combine_rule(value: &str) -> Result<PhysicsCombineRule, String> {
    match normalized_identifier(value).as_str() {
        "average" => Ok(PhysicsCombineRule::Average),
        "minimum" => Ok(PhysicsCombineRule::Minimum),
        "maximum" => Ok(PhysicsCombineRule::Maximum),
        "multiply" => Ok(PhysicsCombineRule::Multiply),
        _ => Err(format!("unsupported combine rule `{value}`")),
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
