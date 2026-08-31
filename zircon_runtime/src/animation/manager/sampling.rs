use crate::core::framework::animation::AnimationChannelValueAsset;
use crate::core::framework::animation::{AnimationError, AnimationParameterValue, AnimationResult};
use crate::core::math::{Quat, Real, Vec3};

#[cfg(test)]
#[path = "sampling/quaternion_property_tests.rs"]
mod quaternion_property_tests;

pub(super) const DEFAULT_GRAPH_CLIP_PLAYBACK_SPEED: Real = 1.0;

pub(super) fn finite_graph_clip_playback_speed(playback_speed: Real) -> Real {
    if playback_speed.is_finite() {
        playback_speed
    } else {
        DEFAULT_GRAPH_CLIP_PLAYBACK_SPEED
    }
}

pub(super) fn animation_parameter_value_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Vec3(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Vec4(value) => value.iter().all(|component| component.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}

pub(super) fn resolve_sample_time(
    duration_seconds: Real,
    time_seconds: Real,
    looping: bool,
) -> Real {
    if !duration_seconds.is_finite() || duration_seconds <= Real::EPSILON {
        return 0.0;
    }
    if !time_seconds.is_finite() {
        return 0.0;
    }
    let clamped = time_seconds.max(0.0);
    if looping {
        if clamped <= duration_seconds {
            clamped
        } else {
            clamped.rem_euclid(duration_seconds)
        }
    } else {
        clamped.min(duration_seconds)
    }
}

pub(super) fn real_array_is_finite<const N: usize>(value: &[Real; N]) -> bool {
    value.iter().all(|component| component.is_finite())
}

pub(super) fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

pub(super) fn sample_vec3(value: &AnimationChannelValueAsset) -> AnimationResult<Vec3> {
    match value {
        AnimationChannelValueAsset::Vec3(value) if value.iter().all(|c| c.is_finite()) => {
            Ok(Vec3::from_array(*value))
        }
        AnimationChannelValueAsset::Vec3(_) => Err(AnimationError::NonFiniteSample {
            sample_kind: "vec3",
        }),
        other => Err(AnimationError::SampleTypeMismatch {
            expected: "vec3",
            actual: animation_channel_value_kind(other),
        }),
    }
}

pub(super) fn sample_quaternion(value: &AnimationChannelValueAsset) -> AnimationResult<Quat> {
    match value {
        AnimationChannelValueAsset::Quaternion(value) => {
            let (finite, normalizable) = quaternion_sample_properties(value);
            if finite && normalizable {
                Ok(Quat::from_array(*value).normalize())
            } else if finite {
                Err(AnimationError::ZeroLengthQuaternionSample)
            } else {
                Err(AnimationError::NonFiniteSample {
                    sample_kind: "quaternion",
                })
            }
        }
        other => Err(AnimationError::SampleTypeMismatch {
            expected: "quaternion",
            actual: animation_channel_value_kind(other),
        }),
    }
}

fn quaternion_sample_properties(value: &[Real; 4]) -> (bool, bool) {
    let mut finite = true;
    let mut squared_length = 0.0;
    for component in value {
        finite &= component.is_finite();
        squared_length += component * component;
    }
    (finite, squared_length > Real::EPSILON)
}

fn animation_channel_value_kind(value: &AnimationChannelValueAsset) -> &'static str {
    match value {
        AnimationChannelValueAsset::Bool(_) => "bool",
        AnimationChannelValueAsset::Integer(_) => "integer",
        AnimationChannelValueAsset::Scalar(_) => "scalar",
        AnimationChannelValueAsset::Vec2(_) => "vec2",
        AnimationChannelValueAsset::Vec3(_) => "vec3",
        AnimationChannelValueAsset::Vec4(_) => "vec4",
        AnimationChannelValueAsset::Quaternion(_) => "quaternion",
    }
}
