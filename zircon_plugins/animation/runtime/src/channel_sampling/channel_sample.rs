//! 管理器同步预览从原始作者通道取样；生产帧的剪辑评估器另在编译和校验后采样。
//! 直接调用时须留意原始键序与数值有效性由上层资产契约保证。
//! Authored animation-channel key selection and interpolation dispatch.

use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
use zircon_runtime::core::math::{Quat, Real};

use super::interpolation::sample_hermite;

pub(crate) trait AnimationChannelSampleExt {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

impl AnimationChannelSampleExt for AnimationChannelAsset {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() {
            return None;
        }

        let first = self.keys.first()?;
        let mut sampled_pair = None;
        let mut previous = first;
        for key in self.keys.iter().skip(1) {
            if !previous.time_seconds.is_finite() {
                return None;
            }
            if sampled_pair.is_none()
                && time_seconds >= previous.time_seconds
                && time_seconds <= key.time_seconds
            {
                sampled_pair = Some((previous, key));
            }
            previous = key;
        }
        if !previous.time_seconds.is_finite() {
            return None;
        }

        if self.keys.len() == 1 || time_seconds <= first.time_seconds {
            return Some(first.value.clone());
        }
        let last = previous;
        if time_seconds >= last.time_seconds {
            return Some(last.value.clone());
        }

        let Some((left, right)) = sampled_pair else {
            return Some(last.value.clone());
        };
        Some(match self.interpolation {
            AnimationInterpolationAsset::Step => left.value.clone(),
            AnimationInterpolationAsset::Linear => sample_linear(left, right, time_seconds),
            AnimationInterpolationAsset::Hermite => sample_hermite(left, right, time_seconds),
        })
    }
}

fn sample_linear(
    left: &AnimationChannelKeyAsset,
    right: &AnimationChannelKeyAsset,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds - left.time_seconds).max(Real::EPSILON);
    let t = ((time_seconds - left.time_seconds) / duration).clamp(0.0, 1.0);
    match (&left.value, &right.value) {
        (
            AnimationChannelValueAsset::Scalar(left_value),
            AnimationChannelValueAsset::Scalar(right_value),
        ) => AnimationChannelValueAsset::Scalar(lerp(*left_value, *right_value, t)),
        (
            AnimationChannelValueAsset::Vec2(left_value),
            AnimationChannelValueAsset::Vec2(right_value),
        ) => AnimationChannelValueAsset::Vec2(lerp_array(left_value, right_value, t)),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(lerp_array(left_value, right_value, t)),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(lerp_array(left_value, right_value, t)),
        (
            AnimationChannelValueAsset::Quaternion(left_value),
            AnimationChannelValueAsset::Quaternion(right_value),
        ) => {
            let left_quat = Quat::from_array(*left_value).normalize();
            let right_quat = Quat::from_array(*right_value).normalize();
            AnimationChannelValueAsset::Quaternion(
                left_quat.slerp(right_quat, t).normalize().to_array(),
            )
        }
        _ => left.value.clone(),
    }
}

fn lerp(left: Real, right: Real, t: Real) -> Real {
    left + (right - left) * t
}

fn lerp_array<const N: usize>(left: &[Real; N], right: &[Real; N], t: Real) -> [Real; N] {
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        result[index] = lerp(left[index], right[index], t);
        index += 1;
    }
    result
}

#[cfg(test)]
#[path = "tests/channel_sample.rs"]
mod tests;
