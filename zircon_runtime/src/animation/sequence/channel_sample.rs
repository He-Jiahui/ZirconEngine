use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
use crate::core::math::{Quat, Real};

use super::interpolation::sample_hermite;

pub(crate) trait AnimationChannelSampleExt {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

impl AnimationChannelSampleExt for AnimationChannelAsset {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() || self.keys.iter().any(|key| !key.time_seconds.is_finite()) {
            return None;
        }

        let first = self.keys.first()?;
        if self.keys.len() == 1 || time_seconds <= first.time_seconds {
            return Some(first.value.clone());
        }
        let last = self.keys.last()?;
        if time_seconds >= last.time_seconds {
            return Some(last.value.clone());
        }

        // The channel boundary checks above establish a valid interior sample.
        // Exact key times remain in the preceding interval, matching Step's
        // established hold behavior while locating that interval logarithmically.
        let right_index = self
            .keys
            .partition_point(|key| key.time_seconds < time_seconds);
        let left = &self.keys[right_index - 1];
        let right = &self.keys[right_index];
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
mod tests {
    use super::*;

    #[test]
    fn linear_interpolation_samples_midpoint_vec3_values() {
        let channel = AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Linear,
            keys: vec![
                key(0.0, AnimationChannelValueAsset::Vec3([0.0, 2.0, 4.0])),
                key(2.0, AnimationChannelValueAsset::Vec3([10.0, 6.0, 8.0])),
            ],
        };

        let sample = channel.sample(1.0).expect("midpoint sample");

        assert_eq!(sample, AnimationChannelValueAsset::Vec3([5.0, 4.0, 6.0]));
    }

    #[test]
    fn linear_interpolation_slerps_quaternion_values() {
        let target = Quat::from_rotation_y(std::f32::consts::PI);
        let channel = AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Linear,
            keys: vec![
                key(
                    0.0,
                    AnimationChannelValueAsset::Quaternion(Quat::IDENTITY.to_array()),
                ),
                key(
                    2.0,
                    AnimationChannelValueAsset::Quaternion(target.to_array()),
                ),
            ],
        };

        let sample = channel.sample(1.0).expect("midpoint sample");
        let AnimationChannelValueAsset::Quaternion(value) = sample else {
            panic!("expected quaternion sample");
        };
        let midpoint = Quat::from_array(value);
        let expected = Quat::IDENTITY.slerp(target, 0.5).normalize();

        assert!((midpoint.length() - 1.0).abs() < 0.0001);
        assert!(midpoint.abs_diff_eq(expected, 0.0001));
    }

    #[test]
    fn step_interpolation_keeps_the_preceding_value_at_an_exact_interior_key() {
        let channel = AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Step,
            keys: vec![
                key(0.0, AnimationChannelValueAsset::Scalar(1.0)),
                key(1.0, AnimationChannelValueAsset::Scalar(2.0)),
                key(2.0, AnimationChannelValueAsset::Scalar(3.0)),
            ],
        };

        assert_eq!(
            channel.sample(1.0),
            Some(AnimationChannelValueAsset::Scalar(1.0))
        );
        assert_eq!(
            channel.sample(1.000_1),
            Some(AnimationChannelValueAsset::Scalar(2.0))
        );
    }

    fn key(time_seconds: Real, value: AnimationChannelValueAsset) -> AnimationChannelKeyAsset {
        AnimationChannelKeyAsset {
            time_seconds,
            value,
            in_tangent: None,
            out_tangent: None,
        }
    }
}
