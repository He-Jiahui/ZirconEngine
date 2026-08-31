use std::str::FromStr;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::physics::PhysicsCombineRule;
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::resource::{ResourceHandle, ResourceId, ResourceMarker};
use crate::scene::components::{JointKind, Mobility, RigidBodyType};
use crate::scene::{SceneError, SceneResult};

use super::errors::unknown_property_error;
use super::identifiers::normalized_identifier_matches;
use super::values::{expect_bool, expect_scalar};

pub(in crate::scene::world::property_access) fn expect_resource_id(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<ResourceId> {
    let ScenePropertyValue::Resource(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "resource id".to_string(),
        });
    };
    match ResourceId::from_str(&value) {
        Ok(resource_id) => Ok(resource_id),
        Err(error) => Err(SceneError::InvalidPropertyResourceId {
            property_path: property_path.to_string(),
            source_message: error.to_string(),
        }),
    }
}

pub(in crate::scene::world::property_access) fn expect_animation_parameter(
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
) -> SceneResult<AnimationParameterValue> {
    let ScenePropertyValue::AnimationParameter(value) = value else {
        return Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_string(),
            expected: "animation parameter".to_string(),
        });
    };
    Ok(value)
}

pub(in crate::scene::world::property_access) fn parse_mobility(
    value: &str,
) -> SceneResult<Mobility> {
    if normalized_identifier_matches(value, "dynamic") {
        Ok(Mobility::Dynamic)
    } else if normalized_identifier_matches(value, "static") {
        Ok(Mobility::Static)
    } else {
        Err(SceneError::UnsupportedPropertyValue {
            kind: "mobility",
            value: value.to_string(),
        })
    }
}

pub(in crate::scene::world::property_access) fn parse_rigid_body_type(
    value: &str,
) -> SceneResult<RigidBodyType> {
    if normalized_identifier_matches(value, "dynamic") {
        Ok(RigidBodyType::Dynamic)
    } else if normalized_identifier_matches(value, "static") {
        Ok(RigidBodyType::Static)
    } else if normalized_identifier_matches(value, "kinematic") {
        Ok(RigidBodyType::Kinematic)
    } else {
        Err(SceneError::UnsupportedPropertyValue {
            kind: "rigid body type",
            value: value.to_string(),
        })
    }
}

pub(in crate::scene::world::property_access) fn parse_joint_kind(
    value: &str,
) -> SceneResult<JointKind> {
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
        Err(SceneError::UnsupportedPropertyValue {
            kind: "joint kind",
            value: value.to_string(),
        })
    }
}

pub(in crate::scene::world::property_access) fn parse_combine_rule(
    value: &str,
) -> SceneResult<PhysicsCombineRule> {
    if normalized_identifier_matches(value, "average") {
        Ok(PhysicsCombineRule::Average)
    } else if normalized_identifier_matches(value, "minimum") {
        Ok(PhysicsCombineRule::Minimum)
    } else if normalized_identifier_matches(value, "maximum") {
        Ok(PhysicsCombineRule::Maximum)
    } else if normalized_identifier_matches(value, "multiply") {
        Ok(PhysicsCombineRule::Multiply)
    } else {
        Err(SceneError::UnsupportedPropertyValue {
            kind: "combine rule",
            value: value.to_string(),
        })
    }
}

pub(in crate::scene::world::property_access) fn combine_rule_label(
    rule: PhysicsCombineRule,
) -> &'static str {
    match rule {
        PhysicsCombineRule::Average => "average",
        PhysicsCombineRule::Minimum => "minimum",
        PhysicsCombineRule::Maximum => "maximum",
        PhysicsCombineRule::Multiply => "multiply",
    }
}

pub(in crate::scene::world::property_access) fn set_animation_player_like_property<TMarker>(
    segments: &[String],
    value: ScenePropertyValue,
    property_path: &ComponentPropertyPath,
    handle: &mut ResourceHandle<TMarker>,
    playback_speed: &mut f32,
    time_seconds: &mut f32,
    weight: Option<&mut f32>,
    looping: &mut bool,
    playing: &mut bool,
) -> SceneResult<bool>
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
