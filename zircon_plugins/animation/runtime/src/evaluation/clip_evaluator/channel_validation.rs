use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelValueAsset, AnimationClipAsset,
};
use zircon_runtime::core::math::{Quat, Real};

use super::{AnimationChannelDataRole, AnimationEvaluationError, AnimationTransformChannel};

pub(super) fn validate_clip_channels(
    clip: &AnimationClipAsset,
) -> Result<(), AnimationEvaluationError> {
    for (track_index, track) in clip.tracks.iter().enumerate() {
        validate_channel(
            track_index,
            AnimationTransformChannel::Translation,
            &track.translation,
        )?;
        validate_channel(
            track_index,
            AnimationTransformChannel::Rotation,
            &track.rotation,
        )?;
        validate_channel(track_index, AnimationTransformChannel::Scale, &track.scale)?;
    }
    Ok(())
}

fn validate_channel(
    track_index: usize,
    channel: AnimationTransformChannel,
    source: &AnimationChannelAsset,
) -> Result<(), AnimationEvaluationError> {
    let mut previous_time = None;
    for (key_index, key) in source.keys.iter().enumerate() {
        if !key.time_seconds.is_finite() {
            return Err(AnimationEvaluationError::NonFiniteChannelTime {
                track_index,
                channel,
                key_index,
            });
        }
        if previous_time.is_some_and(|previous| key.time_seconds <= previous) {
            return Err(AnimationEvaluationError::NonIncreasingChannelTime {
                track_index,
                channel,
                previous_key_index: key_index - 1,
                key_index,
            });
        }
        previous_time = Some(key.time_seconds);

        validate_channel_value(
            track_index,
            channel,
            key_index,
            AnimationChannelDataRole::Value,
            &key.value,
        )?;
        if let Some(tangent) = key.in_tangent.as_ref() {
            validate_channel_value(
                track_index,
                channel,
                key_index,
                AnimationChannelDataRole::InTangent,
                tangent,
            )?;
        }
        if let Some(tangent) = key.out_tangent.as_ref() {
            validate_channel_value(
                track_index,
                channel,
                key_index,
                AnimationChannelDataRole::OutTangent,
                tangent,
            )?;
        }
    }
    Ok(())
}

fn validate_channel_value(
    track_index: usize,
    channel: AnimationTransformChannel,
    key_index: usize,
    role: AnimationChannelDataRole,
    value: &AnimationChannelValueAsset,
) -> Result<(), AnimationEvaluationError> {
    if !value_matches_channel(value, channel) {
        return Err(AnimationEvaluationError::InvalidChannelValueType {
            track_index,
            channel,
            key_index,
            role,
        });
    }
    if !channel_value_is_finite(value) {
        return Err(AnimationEvaluationError::NonFiniteChannelValue {
            track_index,
            channel,
            key_index,
            role,
        });
    }
    if role == AnimationChannelDataRole::Value
        && channel == AnimationTransformChannel::Rotation
        && matches!(value, AnimationChannelValueAsset::Quaternion(rotation)
            if Quat::from_array(*rotation).length_squared() <= Real::EPSILON)
    {
        return Err(AnimationEvaluationError::ZeroLengthChannelRotation {
            track_index,
            key_index,
        });
    }
    Ok(())
}

fn value_matches_channel(
    value: &AnimationChannelValueAsset,
    channel: AnimationTransformChannel,
) -> bool {
    matches!(
        (channel, value),
        (
            AnimationTransformChannel::Translation | AnimationTransformChannel::Scale,
            AnimationChannelValueAsset::Vec3(_)
        ) | (
            AnimationTransformChannel::Rotation,
            AnimationChannelValueAsset::Quaternion(_)
        )
    )
}

fn channel_value_is_finite(value: &AnimationChannelValueAsset) -> bool {
    match value {
        AnimationChannelValueAsset::Bool(_) | AnimationChannelValueAsset::Integer(_) => true,
        AnimationChannelValueAsset::Scalar(value) => value.is_finite(),
        AnimationChannelValueAsset::Vec2(value) => value.iter().all(|value| value.is_finite()),
        AnimationChannelValueAsset::Vec3(value) => value.iter().all(|value| value.is_finite()),
        AnimationChannelValueAsset::Vec4(value) | AnimationChannelValueAsset::Quaternion(value) => {
            value.iter().all(|value| value.is_finite())
        }
    }
}
