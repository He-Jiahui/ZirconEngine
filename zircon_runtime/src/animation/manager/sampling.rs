use crate::asset::AnimationChannelValueAsset;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::{Quat, Real, Vec3};

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

pub(super) fn sample_vec3(value: &AnimationChannelValueAsset) -> Result<Vec3, String> {
    match value {
        AnimationChannelValueAsset::Vec3(value) if value.iter().all(|c| c.is_finite()) => {
            Ok(Vec3::from_array(*value))
        }
        AnimationChannelValueAsset::Vec3(value) => {
            Err(format!("non-finite vec3 animation sample: {value:?}"))
        }
        other => Err(format!("expected vec3 animation sample, found {other:?}")),
    }
}

pub(super) fn sample_quaternion(value: &AnimationChannelValueAsset) -> Result<Quat, String> {
    match value {
        AnimationChannelValueAsset::Quaternion(value)
            if value.iter().all(|c| c.is_finite()) && quaternion_array_is_normalizable(value) =>
        {
            Ok(Quat::from_array(*value).normalize())
        }
        AnimationChannelValueAsset::Quaternion(value) if value.iter().all(|c| c.is_finite()) => {
            Err(format!(
                "zero-length quaternion animation sample: {value:?}"
            ))
        }
        AnimationChannelValueAsset::Quaternion(value) => {
            Err(format!("non-finite quaternion animation sample: {value:?}"))
        }
        other => Err(format!(
            "expected quaternion animation sample, found {other:?}"
        )),
    }
}
