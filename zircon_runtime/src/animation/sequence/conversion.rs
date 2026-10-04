use crate::core::framework::animation::AnimationChannelValueAsset;
use crate::core::framework::animation::{AnimationError, AnimationResult};
use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::Real;

// 序列采样结果进入世界属性写入器前在这里转换类型；非有限数值由动画层报告，不交给场景层猜测来源。
pub(super) fn scene_property_value_from_channel(
    value: &AnimationChannelValueAsset,
) -> AnimationResult<ScenePropertyValue> {
    match value {
        AnimationChannelValueAsset::Bool(value) => Ok(ScenePropertyValue::Bool(*value)),
        AnimationChannelValueAsset::Integer(value) => {
            Ok(ScenePropertyValue::Integer(*value as i64))
        }
        AnimationChannelValueAsset::Vec2(value) => {
            project_finite_array(value, "vec2", ScenePropertyValue::Vec2)
        }
        AnimationChannelValueAsset::Vec3(value) => {
            project_finite_array(value, "vec3", ScenePropertyValue::Vec3)
        }
        AnimationChannelValueAsset::Vec4(value) => {
            project_finite_array(value, "vec4", ScenePropertyValue::Vec4)
        }
        AnimationChannelValueAsset::Scalar(value) => {
            if value.is_finite() {
                Ok(ScenePropertyValue::Scalar(*value))
            } else {
                Err(non_finite_channel_sample("scalar"))
            }
        }
        AnimationChannelValueAsset::Quaternion(value) => {
            if !value.iter().all(|component| component.is_finite()) {
                return Err(non_finite_channel_sample("quaternion"));
            }
            if !quaternion_array_is_normalizable(value) {
                return Err(AnimationError::ZeroLengthQuaternionChannelSample);
            }
            Ok(ScenePropertyValue::Quaternion(*value))
        }
    }
}

fn project_finite_array<const N: usize>(
    value: &[Real; N],
    sample_kind: &'static str,
    project: impl FnOnce([Real; N]) -> ScenePropertyValue,
) -> AnimationResult<ScenePropertyValue> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(project(*value))
    } else {
        Err(non_finite_channel_sample(sample_kind))
    }
}

fn non_finite_channel_sample(sample_kind: &'static str) -> AnimationError {
    AnimationError::NonFiniteChannelSample { sample_kind }
}

fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

#[cfg(test)]
#[path = "tests/conversion_optimization_tests.rs"]
mod optimization_tests;
