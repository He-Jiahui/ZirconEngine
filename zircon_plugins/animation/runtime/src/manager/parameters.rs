use zircon_runtime::core::framework::animation::AnimationGraphAsset;
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};
use zircon_runtime::core::math::Real;

use super::sampling::animation_parameter_value_is_finite;

#[cfg(test)]
#[path = "parameters/performance_tests.rs"]
mod optimization_batch_20260830cr_tests;

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

pub(super) fn set_parameter(
    parameters: &mut AnimationParameterMap,
    name: &str,
    value: AnimationParameterValue,
) {
    if animation_parameter_value_is_finite(&value) {
        if let Some(parameter) = parameters.get_mut(name) {
            *parameter = value;
        } else {
            parameters.insert(name.to_string(), value);
        }
    }
}

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
