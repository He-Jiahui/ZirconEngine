use zircon_runtime::core::framework::ai::{AiDecisionStatus, AiPerceptionSense};

pub(super) const TASK_RESULT_PARAMETER_KEY: &str = "result";
pub(super) const PARALLEL_SUCCESS_POLICY_PARAMETER_KEY: &str = "success_policy";
pub(super) const PARALLEL_FAILURE_POLICY_PARAMETER_KEY: &str = "failure_policy";
pub(super) const BLACKBOARD_KEY_PARAMETER_KEY: &str = "blackboard_key";
pub(super) const BLACKBOARD_EXISTS_PARAMETER_KEY: &str = "exists";
pub(super) const BLACKBOARD_INVERT_PARAMETER_KEY: &str = "invert";
pub(super) const SUBTREE_TARGET_PARAMETER_KEY: &str = "behavior_tree";
pub(super) const PERCEPTION_SENSE_PARAMETER_KEY: &str = "perception_sense";
pub(super) const PERCEPTION_SOURCE_PARAMETER_KEY: &str = "perception_source";
pub(super) const PERCEPTION_MIN_STRENGTH_PARAMETER_KEY: &str = "perception_min_strength";
pub(super) const PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY: &str = "perception_max_age_seconds";
pub(super) const PERCEPTION_EXISTS_PARAMETER_KEY: &str = "perception_exists";

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

pub(super) const PERCEPTION_CONDITION_PARAMETER_KEYS: &[&str] = &[
    PERCEPTION_SENSE_PARAMETER_KEY,
    PERCEPTION_SOURCE_PARAMETER_KEY,
    PERCEPTION_MIN_STRENGTH_PARAMETER_KEY,
    PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    PERCEPTION_EXISTS_PARAMETER_KEY,
];

pub(super) const DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS: &[&str] = &[
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

pub(super) fn parse_task_result(value: &str) -> Option<AiDecisionStatus> {
    match normalized_parameter_value(value).as_str() {
        "idle" => Some(AiDecisionStatus::Idle),
        "running" | "in_progress" | "inprogress" => Some(AiDecisionStatus::Running),
        "succeeded" | "success" | "succeed" => Some(AiDecisionStatus::Succeeded),
        "failed" | "failure" | "fail" => Some(AiDecisionStatus::Failed),
        "blocked" => Some(AiDecisionStatus::Blocked),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ParallelPolicy {
    All,
    Any,
}

pub(super) fn parse_parallel_policy(value: &str) -> Option<ParallelPolicy> {
    match normalized_parameter_value(value).as_str() {
        "all" => Some(ParallelPolicy::All),
        "any" => Some(ParallelPolicy::Any),
        _ => None,
    }
}

pub(super) fn parse_perception_sense(value: &str) -> Option<AiPerceptionSense> {
    match normalized_parameter_value(value).as_str() {
        "sight" | "see" | "vision" => Some(AiPerceptionSense::Sight),
        "hearing" | "hear" | "sound" => Some(AiPerceptionSense::Hearing),
        "damage" => Some(AiPerceptionSense::Damage),
        "touch" => Some(AiPerceptionSense::Touch),
        "custom" => Some(AiPerceptionSense::Custom),
        _ => None,
    }
}

fn normalized_parameter_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
