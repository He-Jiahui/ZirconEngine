use zircon_runtime::core::framework::animation::AnimationChannelValueAsset;
use zircon_runtime::core::framework::scene::ScenePropertyValue;
use zircon_runtime::core::math::Real;

pub(super) fn scene_property_value_from_channel(
    value: &AnimationChannelValueAsset,
) -> Result<ScenePropertyValue, String> {
    if !animation_channel_value_is_finite(value) {
        return Err(format!("non-finite animation channel sample: {value:?}"));
    }
    if let AnimationChannelValueAsset::Quaternion(value) = value {
        if !quaternion_array_is_normalizable(value) {
            return Err(format!(
                "zero-length quaternion animation channel sample: {value:?}"
            ));
        }
    }

    Ok(match value {
        AnimationChannelValueAsset::Bool(value) => ScenePropertyValue::Bool(*value),
        AnimationChannelValueAsset::Integer(value) => ScenePropertyValue::Integer(*value as i64),
        AnimationChannelValueAsset::Scalar(value) => ScenePropertyValue::Scalar(*value),
        AnimationChannelValueAsset::Vec2(value) => ScenePropertyValue::Vec2(*value),
        AnimationChannelValueAsset::Vec3(value) => ScenePropertyValue::Vec3(*value),
        AnimationChannelValueAsset::Vec4(value) => ScenePropertyValue::Vec4(*value),
        AnimationChannelValueAsset::Quaternion(value) => ScenePropertyValue::Quaternion(*value),
    })
}

fn animation_channel_value_is_finite(value: &AnimationChannelValueAsset) -> bool {
    match value {
        AnimationChannelValueAsset::Bool(_) | AnimationChannelValueAsset::Integer(_) => true,
        AnimationChannelValueAsset::Scalar(value) => value.is_finite(),
        AnimationChannelValueAsset::Vec2(value) => {
            value.iter().all(|component| component.is_finite())
        }
        AnimationChannelValueAsset::Vec3(value) => {
            value.iter().all(|component| component.is_finite())
        }
        AnimationChannelValueAsset::Vec4(value) | AnimationChannelValueAsset::Quaternion(value) => {
            value.iter().all(|component| component.is_finite())
        }
    }
}

fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}
