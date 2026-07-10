use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
use zircon_runtime::core::math::{Quat, Real};

use super::hermite::sample_hermite;

pub(super) fn sample_channel(
    channel: &AnimationChannelAsset,
    time_seconds: Real,
) -> Option<AnimationChannelValueAsset> {
    if !time_seconds.is_finite() || channel.keys.iter().any(|key| !key.time_seconds.is_finite()) {
        return None;
    }

    let first = channel.keys.first()?;
    if channel.keys.len() == 1 || time_seconds <= first.time_seconds {
        return Some(first.value.clone());
    }
    let last = channel.keys.last()?;
    if time_seconds >= last.time_seconds {
        return Some(last.value.clone());
    }

    for pair in channel.keys.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        if !(left.time_seconds..=right.time_seconds).contains(&time_seconds) {
            continue;
        }
        return Some(match channel.interpolation {
            AnimationInterpolationAsset::Step => left.value.clone(),
            AnimationInterpolationAsset::Linear => sample_linear(left, right, time_seconds),
            AnimationInterpolationAsset::Hermite => sample_hermite(left, right, time_seconds),
        });
    }

    Some(last.value.clone())
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
