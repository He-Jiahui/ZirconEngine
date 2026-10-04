use crate::core::framework::animation::compiler::sequence::{
    AnimationCompiledSequenceKey, AnimationCompiledSequenceTrack,
};
use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
use crate::core::math::{Quat, Real};

use super::interpolation::sample_hermite;

pub(crate) trait AnimationChannelSampleExt {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

pub(crate) trait AnimationCompiledSequenceTrackSampleExt {
    fn sample_compiled(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

impl AnimationChannelSampleExt for AnimationChannelAsset {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() || self.keys.iter().any(|key| !key.time_seconds.is_finite()) {
            return None;
        }

        sample_channel_keys(self.interpolation, &self.keys, time_seconds)
    }
}

impl AnimationCompiledSequenceTrackSampleExt for AnimationCompiledSequenceTrack {
    fn sample_compiled(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() {
            return None;
        }

        sample_channel_keys(self.interpolation(), self.keys(), time_seconds)
    }
}

trait AnimationChannelKeyView {
    fn time_seconds(&self) -> Real;
    fn value(&self) -> &AnimationChannelValueAsset;
    fn in_tangent(&self) -> Option<&AnimationChannelValueAsset>;
    fn out_tangent(&self) -> Option<&AnimationChannelValueAsset>;
}

impl AnimationChannelKeyView for AnimationChannelKeyAsset {
    fn time_seconds(&self) -> Real {
        self.time_seconds
    }

    fn value(&self) -> &AnimationChannelValueAsset {
        &self.value
    }

    fn in_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.in_tangent.as_ref()
    }

    fn out_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.out_tangent.as_ref()
    }
}

impl AnimationChannelKeyView for AnimationCompiledSequenceKey {
    fn time_seconds(&self) -> Real {
        self.time_seconds()
    }

    fn value(&self) -> &AnimationChannelValueAsset {
        self.value()
    }

    fn in_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.in_tangent()
    }

    fn out_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        self.out_tangent()
    }
}

fn sample_channel_keys<K: AnimationChannelKeyView>(
    interpolation: AnimationInterpolationAsset,
    keys: &[K],
    time_seconds: Real,
) -> Option<AnimationChannelValueAsset> {
    let first = keys.first()?;
    if keys.len() == 1 || time_seconds <= first.time_seconds() {
        return Some(first.value().clone());
    }
    let last = keys.last()?;
    if time_seconds >= last.time_seconds() {
        return Some(last.value().clone());
    }

    // Exact key times stay in the preceding interval, preserving Step's hold behavior.
    let right_index = keys.partition_point(|key| key.time_seconds() < time_seconds);
    let left = &keys[right_index - 1];
    let right = &keys[right_index];
    Some(match interpolation {
        AnimationInterpolationAsset::Step => left.value().clone(),
        AnimationInterpolationAsset::Linear => sample_linear(left, right, time_seconds),
        AnimationInterpolationAsset::Hermite => sample_hermite(
            left.time_seconds(),
            left.value(),
            left.out_tangent(),
            right.time_seconds(),
            right.value(),
            right.in_tangent(),
            time_seconds,
        ),
    })
}

fn sample_linear<K: AnimationChannelKeyView>(
    left: &K,
    right: &K,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds() - left.time_seconds()).max(Real::EPSILON);
    let t = ((time_seconds - left.time_seconds()) / duration).clamp(0.0, 1.0);
    match (left.value(), right.value()) {
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
        _ => left.value().clone(),
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
