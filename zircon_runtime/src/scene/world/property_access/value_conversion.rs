use std::str::FromStr;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::physics::PhysicsCombineRule;
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Quat, Real, Vec2, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, ResourceMarker};
use crate::scene::components::{JointKind, Mobility, RigidBodyType};
use crate::scene::{EntityId, SceneError, SceneResult};

use super::super::World;

pub(super) fn expect_segment_count(
    segments: &[String],
    expected: usize,
    property_path: &ComponentPropertyPath,
) -> SceneResult<()> {
    if segments.len() == expected {
        Ok(())
    } else {
        Err(SceneError::PropertySegmentCount {
            property_path: property_path.to_string(),
            expected,
            actual: segments.len(),
        })
    }
}

pub(super) fn expect_segment(
    actual: &str,
    expected: &[&str],
    property_path: &ComponentPropertyPath,
) -> SceneResult<()> {
    for candidate in expected {
        if *candidate == actual {
            return Ok(());
        }
    }

    unknown_property(property_path)
}

fn unknown_property<T>(property_path: &ComponentPropertyPath) -> SceneResult<T> {
    Err(SceneError::UnknownProperty {
        property_path: property_path.to_string(),
    })
}

pub(super) fn unknown_property_error(property_path: &ComponentPropertyPath) -> SceneResult<bool> {
    unknown_property(property_path)
}

pub(super) fn missing_component_error(
    entity: EntityId,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    Err(SceneError::MissingPropertyComponent {
        entity,
        property_path: property_path.to_string(),
    })
}

pub(super) fn property_type_error<T>(
    property_path: &ComponentPropertyPath,
    expected: &str,
) -> SceneResult<T> {
    Err(SceneError::PropertyTypeMismatch {
        property_path: property_path.to_string(),
        expected: format!("value of type {expected}"),
    })
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

impl World {
    /// Canonicalizes a component-property DTO once before Runtime interns it.
    pub(in crate::scene::world) fn canonical_component_field_key(
        property_path: &ComponentPropertyPath,
    ) -> String {
        let Some(component) = canonical_runtime_property_component(property_path.component())
        else {
            // Dynamic type identifiers are schema keys and can be case-sensitive.
            // Keep them exact so different runtime schemas never intern to one field ID.
            return property_path.as_str().to_string();
        };
        let segments = property_path.property_segments();
        let mut key = String::with_capacity(
            component.len() + segments.iter().map(String::len).sum::<usize>() + segments.len(),
        );
        key.push_str(&component);
        for segment in segments {
            key.push('.');
            key.push_str(&normalized_identifier(segment));
        }
        key
    }

    pub(in crate::scene::world) fn is_runtime_property_component(
        property_path: &ComponentPropertyPath,
    ) -> bool {
        canonical_runtime_property_component(property_path.component()).is_some()
    }

    pub(in crate::scene::world) fn compiled_property_expect_scalar(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Real> {
        expect_scalar(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_i32(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<i32> {
        expect_i32(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_bool(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        expect_bool(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec2(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec2> {
        expect_vec2(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec3(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec3> {
        expect_vec3(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec4(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec4> {
        expect_vec4(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_quat(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Quat> {
        expect_quat(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_validate_quat_array(
        value: [Real; 4],
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<()> {
        validate_quat_array(value, property_path)
    }
}

fn canonical_runtime_property_component(component: &str) -> Option<&'static str> {
    match normalized_identifier(component).as_str() {
        "light" | "directionallight" => Some("directionallight"),
        "mesh" | "meshrenderer" => Some("meshrenderer"),
        "renderlayermask" | "renderlayer" => Some("renderlayer"),
        "name" => Some("name"),
        "hierarchy" => Some("hierarchy"),
        "transform" => Some("transform"),
        "camera" => Some("camera"),
        "ambientlight" => Some("ambientlight"),
        "pointlight" => Some("pointlight"),
        "rectlight" => Some("rectlight"),
        "spotlight" => Some("spotlight"),
        "rigidbody" => Some("rigidbody"),
        "collider" => Some("collider"),
        "joint" => Some("joint"),
        "animationskeleton" => Some("animationskeleton"),
        "animationplayer" => Some("animationplayer"),
        "animationsequenceplayer" => Some("animationsequenceplayer"),
        "animationgraphplayer" => Some("animationgraphplayer"),
        "animationstatemachineplayer" => Some("animationstatemachineplayer"),
        _ => None,
    }
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

pub(super) fn axis_index(axis: &str, property_path: &ComponentPropertyPath) -> SceneResult<usize> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        _ => Err(SceneError::UnknownPropertyAxis {
            property_path: property_path.to_string(),
            axis_kind: "axis",
        }),
    }
}

pub(super) fn quat_axis_index(
    axis: &str,
    property_path: &ComponentPropertyPath,
) -> SceneResult<usize> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        "w" | "3" => Ok(3),
        _ => Err(SceneError::UnknownPropertyAxis {
            property_path: property_path.to_string(),
            axis_kind: "quaternion axis",
        }),
    }
}

pub(super) fn expect_bool(
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

pub(super) fn expect_string(
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

pub(super) fn expect_enum(
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

pub(super) fn expect_scalar(
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

pub(super) fn expect_u32(
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

pub(super) fn expect_i32(
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

pub(super) fn expect_vec3(
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

pub(super) fn expect_vec2(
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

pub(super) fn expect_vec4(
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

pub(super) fn expect_quat(
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

pub(super) fn validate_quat_array(
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

pub(super) fn expect_resource_id(
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

pub(super) fn expect_animation_parameter(
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

pub(super) fn parse_mobility(value: &str) -> SceneResult<Mobility> {
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

pub(super) fn parse_rigid_body_type(value: &str) -> SceneResult<RigidBodyType> {
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

pub(super) fn parse_joint_kind(value: &str) -> SceneResult<JointKind> {
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

pub(super) fn parse_combine_rule(value: &str) -> SceneResult<PhysicsCombineRule> {
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
