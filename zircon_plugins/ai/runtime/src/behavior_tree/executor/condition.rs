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

#[cfg(test)]
#[path = "condition/allocation_tests.rs"]
mod allocation_tests;

pub(super) fn decorator_condition_passes(
    node: &CompiledBehaviorNode,
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
    dense_value: Option<Option<&AiBlackboardValue>>,
) -> bool {
    let passes = raw_blackboard_condition_passes(node, blackboard, dense_value)
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
    dense_value: Option<Option<&AiBlackboardValue>>,
) -> bool {
    let Some(key) = parameter(node, BLACKBOARD_KEY_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    else {
        return true;
    };
    let value = match dense_value {
        Some(value) => value,
        None => blackboard
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value),
    };
    if let Some(expected_exists) = parameter(node, BLACKBOARD_EXISTS_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
    {
        if value.is_some() != expected_exists {
            return false;
        }
        if !has_value_comparison(node) {
            return true;
        }
    }
    let Some(value) = value else {
        return false;
    };
    value_comparison_passes(node, value)
}

fn raw_perception_condition_passes(
    node: &CompiledBehaviorNode,
    perception: Option<&AiPerceptionSnapshot>,
) -> bool {
    let condition = PerceptionCondition::from_node(node);
    if !condition.configured {
        return true;
    }
    let exists = perception
        .map(|snapshot| {
            snapshot
                .stimuli
                .iter()
                .any(|stimulus| condition.matches(stimulus))
        })
        .unwrap_or(false);
    exists == condition.expected_exists
}

struct PerceptionCondition {
    configured: bool,
    sense: Option<zircon_runtime::core::framework::ai::AiPerceptionSense>,
    source: Option<u64>,
    minimum_strength: Option<f32>,
    maximum_age_seconds: Option<f32>,
    expected_exists: bool,
}

impl PerceptionCondition {
    fn from_node(node: &CompiledBehaviorNode) -> Self {
        let mut condition = Self {
            configured: false,
            sense: None,
            source: None,
            minimum_strength: None,
            maximum_age_seconds: None,
            expected_exists: true,
        };
        let mut saw_sense = false;
        let mut saw_source = false;
        let mut saw_minimum_strength = false;
        let mut saw_maximum_age = false;
        let mut saw_expected_exists = false;
        for parameter in node.parameters() {
            let key = parameter.key.as_str();
            if !PERCEPTION_CONDITION_PARAMETER_KEYS.contains(&key) {
                continue;
            }
            condition.configured = true;
            match key {
                PERCEPTION_SENSE_PARAMETER_KEY if !saw_sense => {
                    saw_sense = true;
                    condition.sense = parameter.value.as_string().and_then(parse_perception_sense);
                }
                PERCEPTION_SOURCE_PARAMETER_KEY if !saw_source => {
                    saw_source = true;
                    if let AiBehaviorNodeParameterValue::Entity(source) = &parameter.value {
                        condition.source = Some(*source);
                    }
                }
                PERCEPTION_MIN_STRENGTH_PARAMETER_KEY if !saw_minimum_strength => {
                    saw_minimum_strength = true;
                    if let AiBehaviorNodeParameterValue::Scalar(minimum) = &parameter.value {
                        condition.minimum_strength = Some(*minimum);
                    }
                }
                PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY if !saw_maximum_age => {
                    saw_maximum_age = true;
                    if let AiBehaviorNodeParameterValue::Scalar(maximum) = &parameter.value {
                        condition.maximum_age_seconds = Some(*maximum);
                    }
                }
                PERCEPTION_EXISTS_PARAMETER_KEY if !saw_expected_exists => {
                    saw_expected_exists = true;
                    if let AiBehaviorNodeParameterValue::Bool(expected) = &parameter.value {
                        condition.expected_exists = *expected;
                    }
                }
                _ => {}
            }
        }
        condition
    }

    fn matches(&self, stimulus: &AiPerceptionStimulus) -> bool {
        if self
            .sense
            .is_some_and(|expected| stimulus.sense != expected)
        {
            return false;
        }
        if self
            .source
            .is_some_and(|expected| stimulus.source != expected)
        {
            return false;
        }
        if self
            .minimum_strength
            .is_some_and(|minimum| stimulus.strength < minimum)
        {
            return false;
        }
        if self
            .maximum_age_seconds
            .is_some_and(|maximum| stimulus.age_seconds > maximum)
        {
            return false;
        }
        true
    }
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
