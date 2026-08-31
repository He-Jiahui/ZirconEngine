use zircon_runtime::core::framework::ai::{AiDecisionStatus, AiPerceptionSense};

#[cfg(test)]
#[path = "parameters/allocation_tests.rs"]
mod allocation_tests;

pub(crate) const TASK_RESULT_PARAMETER_KEY: &str = "result";
pub(crate) const MOVE_TARGET_PARAMETER_KEY: &str = "target";
pub(crate) const ANIMATION_PARAMETER_PARAMETER_KEY: &str = "parameter";
pub(crate) const ANIMATION_TRIGGER_PARAMETER_KEY: &str = "trigger";
pub(crate) const ANIMATION_VALUE_PARAMETER_KEY: &str = "value";
pub(crate) const SCRIPT_CALLBACK_PARAMETER_KEY: &str = "callback";
pub(crate) const PARALLEL_SUCCESS_POLICY_PARAMETER_KEY: &str = "success_policy";
pub(crate) const PARALLEL_FAILURE_POLICY_PARAMETER_KEY: &str = "failure_policy";
pub(crate) const BLACKBOARD_KEY_PARAMETER_KEY: &str = "blackboard_key";
pub(crate) const BLACKBOARD_EXISTS_PARAMETER_KEY: &str = "exists";
pub(crate) const BLACKBOARD_INVERT_PARAMETER_KEY: &str = "invert";
pub(crate) use crate::behavior_tree::SUBTREE_TARGET_PARAMETER_KEY;
pub(crate) const PERCEPTION_SENSE_PARAMETER_KEY: &str = "perception_sense";
pub(crate) const PERCEPTION_SOURCE_PARAMETER_KEY: &str = "perception_source";
pub(crate) const PERCEPTION_MIN_STRENGTH_PARAMETER_KEY: &str = "perception_min_strength";
pub(crate) const PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY: &str = "perception_max_age_seconds";
pub(crate) const PERCEPTION_EXISTS_PARAMETER_KEY: &str = "perception_exists";

pub(super) const PARALLEL_POLICY_PARAMETER_KEYS: &[&str] = &[
    PARALLEL_SUCCESS_POLICY_PARAMETER_KEY,
    PARALLEL_FAILURE_POLICY_PARAMETER_KEY,
];

pub(super) const BLACKBOARD_CONDITION_PARAMETER_KEYS: &[&str] = &[
    BLACKBOARD_EXISTS_PARAMETER_KEY,
    "equals_bool",
    "equals_string",
    "equals_integer",
    "equals_scalar",
    "equals_vec3",
    "equals_entity",
    "greater_than_integer",
    "greater_or_equal_integer",
    "less_than_integer",
    "less_or_equal_integer",
    "greater_than_scalar",
    "greater_or_equal_scalar",
    "less_than_scalar",
    "less_or_equal_scalar",
];

pub(crate) const PERCEPTION_CONDITION_PARAMETER_KEYS: &[&str] = &[
    PERCEPTION_SENSE_PARAMETER_KEY,
    PERCEPTION_SOURCE_PARAMETER_KEY,
    PERCEPTION_MIN_STRENGTH_PARAMETER_KEY,
    PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    PERCEPTION_EXISTS_PARAMETER_KEY,
];

pub(crate) const DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS: &[&str] = &[
    "equals_bool",
    "equals_string",
    "equals_integer",
    "equals_scalar",
    "equals_vec3",
    "equals_entity",
    "greater_than_integer",
    "greater_or_equal_integer",
    "less_than_integer",
    "less_or_equal_integer",
    "greater_than_scalar",
    "greater_or_equal_scalar",
    "less_than_scalar",
    "less_or_equal_scalar",
];

pub(super) const DECORATOR_PARAMETER_KEYS: &[&str] = &[
    BLACKBOARD_KEY_PARAMETER_KEY,
    BLACKBOARD_EXISTS_PARAMETER_KEY,
    BLACKBOARD_INVERT_PARAMETER_KEY,
    "equals_bool",
    "equals_string",
    "equals_integer",
    "equals_scalar",
    "equals_vec3",
    "equals_entity",
    "greater_than_integer",
    "greater_or_equal_integer",
    "less_than_integer",
    "less_or_equal_integer",
    "greater_than_scalar",
    "greater_or_equal_scalar",
    "less_than_scalar",
    "less_or_equal_scalar",
    PERCEPTION_SENSE_PARAMETER_KEY,
    PERCEPTION_SOURCE_PARAMETER_KEY,
    PERCEPTION_MIN_STRENGTH_PARAMETER_KEY,
    PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    PERCEPTION_EXISTS_PARAMETER_KEY,
];

pub(super) const TASK_RESULT_EXPECTED_VALUES: &str =
    "`idle`, `running`, `succeeded`, `failed`, or `blocked`";
pub(super) const PARALLEL_POLICY_EXPECTED_VALUES: &str = "`all` or `any`";
pub(super) const SUBTREE_TARGET_EXPECTED_VALUES: &str =
    "the id of a previously registered behavior tree";
pub(super) const PERCEPTION_SENSE_EXPECTED_VALUES: &str =
    "`sight`, `hearing`, `damage`, `touch`, or `custom`";
pub(super) const NON_NEGATIVE_SCALAR_EXPECTED_VALUE: &str = "a non-negative scalar";

pub(crate) fn parse_task_result(value: &str) -> Option<AiDecisionStatus> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("idle") {
        Some(AiDecisionStatus::Idle)
    } else if matches_ascii_case_insensitive(value, &["running", "in_progress", "inprogress"]) {
        Some(AiDecisionStatus::Running)
    } else if matches_ascii_case_insensitive(value, &["succeeded", "success", "succeed"]) {
        Some(AiDecisionStatus::Succeeded)
    } else if matches_ascii_case_insensitive(value, &["failed", "failure", "fail"]) {
        Some(AiDecisionStatus::Failed)
    } else if value.eq_ignore_ascii_case("blocked") {
        Some(AiDecisionStatus::Blocked)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParallelPolicy {
    All,
    Any,
}

pub(crate) fn parse_parallel_policy(value: &str) -> Option<ParallelPolicy> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("all") {
        Some(ParallelPolicy::All)
    } else if value.eq_ignore_ascii_case("any") {
        Some(ParallelPolicy::Any)
    } else {
        None
    }
}

pub(crate) fn parse_perception_sense(value: &str) -> Option<AiPerceptionSense> {
    let value = value.trim();
    if matches_ascii_case_insensitive(value, &["sight", "see", "vision"]) {
        Some(AiPerceptionSense::Sight)
    } else if matches_ascii_case_insensitive(value, &["hearing", "hear", "sound"]) {
        Some(AiPerceptionSense::Hearing)
    } else if value.eq_ignore_ascii_case("damage") {
        Some(AiPerceptionSense::Damage)
    } else if value.eq_ignore_ascii_case("touch") {
        Some(AiPerceptionSense::Touch)
    } else if value.eq_ignore_ascii_case("custom") {
        Some(AiPerceptionSense::Custom)
    } else {
        None
    }
}

fn matches_ascii_case_insensitive(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
