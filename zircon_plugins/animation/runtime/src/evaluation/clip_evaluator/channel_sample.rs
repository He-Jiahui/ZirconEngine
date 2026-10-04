//! 经编译校验的通道键采样入口；剪辑评估器以有序、有限键值为前提在当前修订内取样。
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
use zircon_runtime::core::math::{Quat, Real};

use super::hermite::sample_hermite;

pub(super) fn sample_channel(
    channel: &AnimationChannelAsset,
    time_seconds: Real,
) -> Option<AnimationChannelValueAsset> {
    if !time_seconds.is_finite() {
        return None;
    }

    // The compiled-clip boundary validates finite, strictly increasing key times once. Sampling
    // therefore selects the interior interval directly instead of revalidating every key or
    // linearly rescanning a stable channel on each frame.
    let first = channel.keys.first()?;
    if channel.keys.len() == 1 || time_seconds <= first.time_seconds {
        return Some(first.value.clone());
    }
    let last = channel.keys.last()?;
    if time_seconds >= last.time_seconds {
        return Some(last.value.clone());
    }

    let (left, right) = interior_key_pair(&channel.keys, time_seconds)?;
    Some(match channel.interpolation {
        AnimationInterpolationAsset::Step => left.value.clone(),
        AnimationInterpolationAsset::Linear => sample_linear(left, right, time_seconds),
        AnimationInterpolationAsset::Hermite => sample_hermite(left, right, time_seconds),
    })
}

fn interior_key_pair(
    keys: &[AnimationChannelKeyAsset],
    time_seconds: Real,
) -> Option<(&AnimationChannelKeyAsset, &AnimationChannelKeyAsset)> {
    let right_index = keys.partition_point(|key| key.time_seconds < time_seconds);
    Some((
        keys.get(right_index.checked_sub(1)?)?,
        keys.get(right_index)?,
    ))
}

fn sample_linear(
    left: &AnimationChannelKeyAsset,
    right: &AnimationChannelKeyAsset,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds - left.time_seconds).max(Real::EPSILON);
    let weight = ((time_seconds - left.time_seconds) / duration).clamp(0.0, 1.0);
    match (&left.value, &right.value) {
        (AnimationChannelValueAsset::Scalar(left), AnimationChannelValueAsset::Scalar(right)) => {
            AnimationChannelValueAsset::Scalar(lerp(*left, *right, weight))
        }
        (AnimationChannelValueAsset::Vec2(left), AnimationChannelValueAsset::Vec2(right)) => {
            AnimationChannelValueAsset::Vec2(lerp_array(left, right, weight))
        }
        (AnimationChannelValueAsset::Vec3(left), AnimationChannelValueAsset::Vec3(right)) => {
            AnimationChannelValueAsset::Vec3(lerp_array(left, right, weight))
        }
        (AnimationChannelValueAsset::Vec4(left), AnimationChannelValueAsset::Vec4(right)) => {
            AnimationChannelValueAsset::Vec4(lerp_array(left, right, weight))
        }
        (
            AnimationChannelValueAsset::Quaternion(left),
            AnimationChannelValueAsset::Quaternion(right),
        ) => AnimationChannelValueAsset::Quaternion(
            Quat::from_array(*left)
                .normalize()
                .slerp(Quat::from_array(*right).normalize(), weight)
                .normalize()
                .to_array(),
        ),
        _ => left.value.clone(),
    }
}

fn lerp(left: Real, right: Real, weight: Real) -> Real {
    left + (right - left) * weight
}

fn lerp_array<const N: usize>(left: &[Real; N], right: &[Real; N], weight: Real) -> [Real; N] {
    std::array::from_fn(|index| lerp(left[index], right[index], weight))
}

#[cfg(test)]
#[path = "tests/channel_sample.rs"]
mod tests;
