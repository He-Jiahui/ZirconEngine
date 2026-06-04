use zircon_runtime::asset::{AnimationChannelKeyAsset, AnimationChannelValueAsset};
use zircon_runtime::core::math::{Quat, Real};

pub(super) fn sample_hermite(
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
        ) => {
            let left_tangent = tangent_scalar(left.out_tangent.as_ref());
            let right_tangent = tangent_scalar(right.in_tangent.as_ref());
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
            tangent_array_2(left.out_tangent.as_ref()),
            right_value,
            tangent_array_2(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(hermite_array(
            left_value,
            tangent_array_3(left.out_tangent.as_ref()),
            right_value,
            tangent_array_3(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(hermite_array(
            left_value,
            tangent_array_4(left.out_tangent.as_ref()),
            right_value,
            tangent_array_4(right.in_tangent.as_ref()),
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
        _ => left.value.clone(),
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
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    h00 * left_value
        + h10 * left_tangent * duration
        + h01 * right_value
        + h11 * right_tangent * duration
}

fn hermite_array<const N: usize>(
    left_value: &[Real; N],
    left_tangent: [Real; N],
    right_value: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    t: Real,
) -> [Real; N] {
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        result[index] = hermite_scalar(
            left_value[index],
            left_tangent[index],
            right_value[index],
            right_tangent[index],
            duration,
            t,
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
