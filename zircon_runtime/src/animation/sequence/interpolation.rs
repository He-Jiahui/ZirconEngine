use crate::core::framework::animation::AnimationChannelValueAsset;
use crate::core::math::{Quat, Real};

pub(super) fn sample_hermite(
    left_time_seconds: Real,
    left_value: &AnimationChannelValueAsset,
    left_out_tangent: Option<&AnimationChannelValueAsset>,
    right_time_seconds: Real,
    right_value: &AnimationChannelValueAsset,
    right_in_tangent: Option<&AnimationChannelValueAsset>,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right_time_seconds - left_time_seconds).max(Real::EPSILON);
    let t = ((time_seconds - left_time_seconds) / duration).clamp(0.0, 1.0);

    match (left_value, right_value) {
        (
            AnimationChannelValueAsset::Scalar(left_value),
            AnimationChannelValueAsset::Scalar(right_value),
        ) => {
            let left_tangent = tangent_scalar(left_out_tangent);
            let right_tangent = tangent_scalar(right_in_tangent);
            AnimationChannelValueAsset::Scalar(hermite_scalar(
                *left_value,
                left_tangent,
                *right_value,
                right_tangent,
                duration,
                t,
            ))
        }
        (
            AnimationChannelValueAsset::Vec2(left_value),
            AnimationChannelValueAsset::Vec2(right_value),
        ) => AnimationChannelValueAsset::Vec2(hermite_array(
            left_value,
            tangent_array_2(left_out_tangent),
            right_value,
            tangent_array_2(right_in_tangent),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(hermite_array(
            left_value,
            tangent_array_3(left_out_tangent),
            right_value,
            tangent_array_3(right_in_tangent),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(hermite_array(
            left_value,
            tangent_array_4(left_out_tangent),
            right_value,
            tangent_array_4(right_in_tangent),
            duration,
            t,
        )),
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
        _ => left_value.clone(),
    }
}

fn hermite_scalar(
    left_value: Real,
    left_tangent: Real,
    right_value: Real,
    right_tangent: Real,
    duration: Real,
    t: Real,
) -> Real {
    HermiteBasis::new(t).sample(
        left_value,
        left_tangent,
        right_value,
        right_tangent,
        duration,
    )
}

#[derive(Clone, Copy)]
struct HermiteBasis {
    h00: Real,
    h10: Real,
    h01: Real,
    h11: Real,
}

impl HermiteBasis {
    fn new(t: Real) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;
        Self {
            h00: 2.0 * t3 - 3.0 * t2 + 1.0,
            h10: t3 - 2.0 * t2 + t,
            h01: -2.0 * t3 + 3.0 * t2,
            h11: t3 - t2,
        }
    }

    fn sample(
        self,
        left_value: Real,
        left_tangent: Real,
        right_value: Real,
        right_tangent: Real,
        duration: Real,
    ) -> Real {
        self.h00 * left_value
            + self.h10 * left_tangent * duration
            + self.h01 * right_value
            + self.h11 * right_tangent * duration
    }
}

fn hermite_array<const N: usize>(
    left_value: &[Real; N],
    left_tangent: [Real; N],
    right_value: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    t: Real,
) -> [Real; N] {
    let basis = HermiteBasis::new(t);
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        result[index] = basis.sample(
            left_value[index],
            left_tangent[index],
            right_value[index],
            right_tangent[index],
            duration,
        );
        index += 1;
    }
    result
}

fn tangent_scalar(value: Option<&AnimationChannelValueAsset>) -> Real {
    match value {
        Some(AnimationChannelValueAsset::Scalar(value)) => *value,
        _ => 0.0,
    }
}

fn tangent_array_2(value: Option<&AnimationChannelValueAsset>) -> [Real; 2] {
    match value {
        Some(AnimationChannelValueAsset::Vec2(value)) => *value,
        _ => [0.0; 2],
    }
}

fn tangent_array_3(value: Option<&AnimationChannelValueAsset>) -> [Real; 3] {
    match value {
        Some(AnimationChannelValueAsset::Vec3(value)) => *value,
        _ => [0.0; 3],
    }
}

fn tangent_array_4(value: Option<&AnimationChannelValueAsset>) -> [Real; 4] {
    match value {
        Some(AnimationChannelValueAsset::Vec4(value)) => *value,
        Some(AnimationChannelValueAsset::Quaternion(value)) => *value,
        _ => [0.0; 4],
    }
}

#[cfg(test)]
#[path = "tests/interpolation_optimization_tests.rs"]
mod optimization_tests;
