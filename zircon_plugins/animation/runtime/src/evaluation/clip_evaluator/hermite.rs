use zircon_runtime::core::framework::animation::{
    AnimationChannelKeyAsset, AnimationChannelValueAsset,
};
use zircon_runtime::core::math::{Quat, Real};

pub(super) fn sample_hermite(
    left: &AnimationChannelKeyAsset,
    right: &AnimationChannelKeyAsset,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds - left.time_seconds).max(Real::EPSILON);
    let weight = ((time_seconds - left.time_seconds) / duration).clamp(0.0, 1.0);
    match (&left.value, &right.value) {
        (
            AnimationChannelValueAsset::Scalar(left_value),
            AnimationChannelValueAsset::Scalar(right_value),
        ) => AnimationChannelValueAsset::Scalar(hermite_scalar(
            *left_value,
            tangent_scalar(left.out_tangent.as_ref()),
            *right_value,
            tangent_scalar(right.in_tangent.as_ref()),
            duration,
            weight,
        )),
        (
            AnimationChannelValueAsset::Vec2(left_value),
            AnimationChannelValueAsset::Vec2(right_value),
        ) => AnimationChannelValueAsset::Vec2(hermite_array(
            left_value,
            tangent_array(left.out_tangent.as_ref()),
            right_value,
            tangent_array(right.in_tangent.as_ref()),
            duration,
            weight,
        )),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(hermite_array(
            left_value,
            tangent_array(left.out_tangent.as_ref()),
            right_value,
            tangent_array(right.in_tangent.as_ref()),
            duration,
            weight,
        )),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(hermite_array(
            left_value,
            tangent_array(left.out_tangent.as_ref()),
            right_value,
            tangent_array(right.in_tangent.as_ref()),
            duration,
            weight,
        )),
        (
            AnimationChannelValueAsset::Quaternion(left_value),
            AnimationChannelValueAsset::Quaternion(right_value),
        ) => AnimationChannelValueAsset::Quaternion(
            Quat::from_array(*left_value)
                .normalize()
                .slerp(Quat::from_array(*right_value).normalize(), weight)
                .normalize()
                .to_array(),
        ),
        _ => left.value.clone(),
    }
}

fn hermite_scalar(
    left: Real,
    left_tangent: Real,
    right: Real,
    right_tangent: Real,
    duration: Real,
    weight: Real,
) -> Real {
    let weight2 = weight * weight;
    let weight3 = weight2 * weight;
    (2.0 * weight3 - 3.0 * weight2 + 1.0) * left
        + (weight3 - 2.0 * weight2 + weight) * left_tangent * duration
        + (-2.0 * weight3 + 3.0 * weight2) * right
        + (weight3 - weight2) * right_tangent * duration
}

fn hermite_array<const N: usize>(
    left: &[Real; N],
    left_tangent: [Real; N],
    right: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    weight: Real,
) -> [Real; N] {
    std::array::from_fn(|index| {
        hermite_scalar(
            left[index],
            left_tangent[index],
            right[index],
            right_tangent[index],
            duration,
            weight,
        )
    })
}

fn tangent_scalar(value: Option<&AnimationChannelValueAsset>) -> Real {
    match value {
        Some(AnimationChannelValueAsset::Scalar(value)) => *value,
        _ => 0.0,
    }
}

fn tangent_array<const N: usize>(value: Option<&AnimationChannelValueAsset>) -> [Real; N] {
    match value {
        Some(AnimationChannelValueAsset::Vec2(value)) if N == 2 => copy_array(value),
        Some(AnimationChannelValueAsset::Vec3(value)) if N == 3 => copy_array(value),
        Some(AnimationChannelValueAsset::Vec4(value)) if N == 4 => copy_array(value),
        Some(AnimationChannelValueAsset::Quaternion(value)) if N == 4 => copy_array(value),
        _ => [0.0; N],
    }
}

fn copy_array<const SOURCE: usize, const TARGET: usize>(source: &[Real; SOURCE]) -> [Real; TARGET] {
    std::array::from_fn(|index| source.get(index).copied().unwrap_or(0.0))
}
