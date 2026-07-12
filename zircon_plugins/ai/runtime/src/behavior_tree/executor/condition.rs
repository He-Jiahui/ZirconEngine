use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameterValue, AiBlackboardEntry, AiBlackboardValue, AiPerceptionSnapshot,
    AiPerceptionStimulus,
};
use zircon_runtime::core::math::Vec3;

use crate::manager::parameters::{
    parse_perception_sense, BLACKBOARD_EXISTS_PARAMETER_KEY, BLACKBOARD_INVERT_PARAMETER_KEY,
    BLACKBOARD_KEY_PARAMETER_KEY, DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS,
    PERCEPTION_CONDITION_PARAMETER_KEYS, PERCEPTION_EXISTS_PARAMETER_KEY,
    PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY, PERCEPTION_MIN_STRENGTH_PARAMETER_KEY,
    PERCEPTION_SENSE_PARAMETER_KEY, PERCEPTION_SOURCE_PARAMETER_KEY,
};

use super::CompiledBehaviorNode;

pub(super) fn decorator_condition_passes(
    node: &CompiledBehaviorNode,
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
) -> bool {
    let passes = raw_blackboard_condition_passes(node, blackboard)
        && raw_perception_condition_passes(node, perception);
    if parameter(node, BLACKBOARD_INVERT_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
        .unwrap_or(false)
    {
        !passes
    } else {
        passes
    }
}

fn raw_blackboard_condition_passes(
    node: &CompiledBehaviorNode,
    blackboard: &[AiBlackboardEntry],
) -> bool {
    let Some(key) = parameter(node, BLACKBOARD_KEY_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    else {
        return true;
    };
    let entry = blackboard.iter().find(|entry| entry.key == key);
    if let Some(expected_exists) = parameter(node, BLACKBOARD_EXISTS_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
    {
        if entry.is_some() != expected_exists {
            return false;
        }
        if !has_value_comparison(node) {
            return true;
        }
    }
    let Some(entry) = entry else {
        return false;
    };
    value_comparison_passes(node, &entry.value)
}

fn raw_perception_condition_passes(
    node: &CompiledBehaviorNode,
    perception: Option<&AiPerceptionSnapshot>,
) -> bool {
    if !has_perception_condition(node) {
        return true;
    }
    let exists = perception
        .map(|snapshot| {
            snapshot
                .stimuli
                .iter()
                .any(|stimulus| perception_stimulus_matches(node, stimulus))
        })
        .unwrap_or(false);
    let expected = parameter(node, PERCEPTION_EXISTS_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
        .unwrap_or(true);
    exists == expected
}

fn has_perception_condition(node: &CompiledBehaviorNode) -> bool {
    node.parameters()
        .iter()
        .any(|parameter| PERCEPTION_CONDITION_PARAMETER_KEYS.contains(&parameter.key.as_str()))
}

fn perception_stimulus_matches(
    node: &CompiledBehaviorNode,
    stimulus: &AiPerceptionStimulus,
) -> bool {
    if let Some(expected) = parameter(node, PERCEPTION_SENSE_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .and_then(parse_perception_sense)
    {
        if stimulus.sense != expected {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Entity(expected)) =
        parameter(node, PERCEPTION_SOURCE_PARAMETER_KEY)
    {
        if stimulus.source != *expected {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(minimum)) =
        parameter(node, PERCEPTION_MIN_STRENGTH_PARAMETER_KEY)
    {
        if stimulus.strength < *minimum {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(maximum)) =
        parameter(node, PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY)
    {
        if stimulus.age_seconds > *maximum {
            return false;
        }
    }
    true
}

fn has_value_comparison(node: &CompiledBehaviorNode) -> bool {
    node.parameters().iter().any(|parameter| {
        DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS.contains(&parameter.key.as_str())
    })
}

fn value_comparison_passes(node: &CompiledBehaviorNode, value: &AiBlackboardValue) -> bool {
    let mut compared = false;
    let mut passed = true;
    macro_rules! compare {
        ($key:literal, $variant:ident, $predicate:expr) => {
            if let Some(AiBehaviorNodeParameterValue::$variant(expected)) = parameter(node, $key) {
                compared = true;
                passed &= matches!(value, AiBlackboardValue::$variant(actual) if ($predicate)(actual, expected));
            }
        };
    }
    compare!("equals_bool", Bool, |actual: &bool, expected: &bool| actual
        == expected);
    compare!(
        "equals_string",
        String,
        |actual: &String, expected: &String| actual == expected
    );
    compare!(
        "equals_integer",
        Integer,
        |actual: &i64, expected: &i64| actual == expected
    );
    compare!(
        "equals_scalar",
        Scalar,
        |actual: &f32, expected: &f32| actual == expected
    );
    compare!("equals_vec3", Vec3, |actual: &Vec3, expected: &Vec3| actual
        == expected);
    compare!(
        "equals_entity",
        Entity,
        |actual: &u64, expected: &u64| actual == expected
    );
    compare!(
        "greater_than_integer",
        Integer,
        |actual: &i64, expected: &i64| actual > expected
    );
    compare!(
        "greater_or_equal_integer",
        Integer,
        |actual: &i64, expected: &i64| actual >= expected
    );
    compare!(
        "less_than_integer",
        Integer,
        |actual: &i64, expected: &i64| actual < expected
    );
    compare!(
        "less_or_equal_integer",
        Integer,
        |actual: &i64, expected: &i64| actual <= expected
    );
    compare!(
        "greater_than_scalar",
        Scalar,
        |actual: &f32, expected: &f32| actual > expected
    );
    compare!(
        "greater_or_equal_scalar",
        Scalar,
        |actual: &f32, expected: &f32| actual >= expected
    );
    compare!(
        "less_than_scalar",
        Scalar,
        |actual: &f32, expected: &f32| actual < expected
    );
    compare!(
        "less_or_equal_scalar",
        Scalar,
        |actual: &f32, expected: &f32| actual <= expected
    );
    !compared || passed
}

fn parameter<'a>(
    node: &'a CompiledBehaviorNode,
    key: &str,
) -> Option<&'a AiBehaviorNodeParameterValue> {
    node.parameters()
        .iter()
        .find(|parameter| parameter.key == key)
        .map(|parameter| &parameter.value)
}
