use crate::core::framework::animation::AnimationGraphAsset;
use crate::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};
use crate::core::math::Real;

use super::sampling::animation_parameter_value_is_finite;

// 图求值以有限默认参数为起点，再叠加调用方覆盖值；非有限值不进入状态判定。
pub(super) fn parameter_defaults(graph: &AnimationGraphAsset) -> AnimationParameterMap {
    graph
        .parameters
        .iter()
        .filter(|parameter| animation_parameter_value_is_finite(&parameter.default_value))
        .map(|parameter| (parameter.name.clone(), parameter.default_value.clone()))
        .collect()
}

pub(super) fn parameter_value(
    parameters: &AnimationParameterMap,
    name: &str,
) -> Option<AnimationParameterValue> {
    parameters.get(name).cloned()
}

// 管理器和状态机共用这份参数表；原位更新保留已有键，缺失键才由调用方名称建表。
pub(super) fn set_parameter(
    parameters: &mut AnimationParameterMap,
    name: &str,
    value: AnimationParameterValue,
) {
    if !animation_parameter_value_is_finite(&value) {
        return;
    }
    if let Some(current) = parameters.get_mut(name) {
        *current = value;
    } else {
        parameters.insert(name.to_string(), value);
    }
}

// 只供状态转移比较读取；调用方必须先确认参数和比较值都是数值类型。
pub(super) fn numeric_parameter(value: Option<&AnimationParameterValue>) -> Real {
    match value {
        Some(AnimationParameterValue::Integer(value)) => *value as Real,
        Some(AnimationParameterValue::Scalar(value)) => *value,
        _ => 0.0,
    }
}

pub(super) fn parameter_scalar(parameters: &AnimationParameterMap, name: &str) -> Option<Real> {
    match parameters.get(name) {
        Some(AnimationParameterValue::Integer(value)) => Some(*value as Real),
        Some(AnimationParameterValue::Scalar(value)) if value.is_finite() => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
#[path = "parameters/tests/in_place_update_tests.rs"]
mod in_place_update_tests;
