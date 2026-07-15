use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeParameterValue, AiManagerError,
};
use zircon_runtime::core::framework::script::ScriptBehaviorCallbackRef;

use super::{
    behavior_node_parameter, expect_string_parameter, expect_vec3_parameter, invalid_parameter,
};
use crate::manager::parameters::{
    ANIMATION_PARAMETER_PARAMETER_KEY, ANIMATION_TRIGGER_PARAMETER_KEY,
    ANIMATION_VALUE_PARAMETER_KEY, MOVE_TARGET_PARAMETER_KEY, SCRIPT_CALLBACK_PARAMETER_KEY,
    TASK_RESULT_PARAMETER_KEY,
};

pub(super) fn validate_integration_node_parameters(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), AiManagerError> {
    let has_placeholder_result = behavior_node_parameter(node, TASK_RESULT_PARAMETER_KEY).is_some();
    match node.implementation.as_str() {
        "move_to" => validate_move_to(tree_id, node, has_placeholder_result)?,
        "play_animation" => validate_play_animation(tree_id, node, has_placeholder_result)?,
        "script_task" => validate_script_task(tree_id, node, has_placeholder_result)?,
        _ => {}
    }
    Ok(())
}

fn validate_move_to(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    has_placeholder_result: bool,
) -> Result<(), AiManagerError> {
    let Some(value) = behavior_node_parameter(node, MOVE_TARGET_PARAMETER_KEY) else {
        if has_placeholder_result {
            return Ok(());
        }
        return missing_parameter(tree_id, node, MOVE_TARGET_PARAMETER_KEY, "vec3");
    };
    expect_vec3_parameter(tree_id, node, MOVE_TARGET_PARAMETER_KEY, value)
}

fn validate_play_animation(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    has_placeholder_result: bool,
) -> Result<(), AiManagerError> {
    let parameter = behavior_node_parameter(node, ANIMATION_PARAMETER_PARAMETER_KEY);
    let trigger = behavior_node_parameter(node, ANIMATION_TRIGGER_PARAMETER_KEY);
    if parameter.is_some() && trigger.is_some() {
        return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: ANIMATION_TRIGGER_PARAMETER_KEY.to_string(),
            expected: "exactly one of `parameter` or `trigger`",
            actual: "both declared".to_string(),
        });
    }
    let Some(name) = parameter.or(trigger) else {
        if has_placeholder_result {
            return Ok(());
        }
        return missing_parameter(
            tree_id,
            node,
            ANIMATION_PARAMETER_PARAMETER_KEY,
            "a string `parameter` or `trigger`",
        );
    };
    let name_key = if parameter.is_some() {
        ANIMATION_PARAMETER_PARAMETER_KEY
    } else {
        ANIMATION_TRIGGER_PARAMETER_KEY
    };
    let name = expect_string_parameter(tree_id, node, name_key, name)?;
    ensure_non_empty_parameter(tree_id, node, name_key, name)?;
    if parameter.is_some() && behavior_node_parameter(node, ANIMATION_VALUE_PARAMETER_KEY).is_none()
    {
        return missing_parameter(
            tree_id,
            node,
            ANIMATION_VALUE_PARAMETER_KEY,
            "bool, i32 integer, scalar, or vec3",
        );
    }
    if trigger.is_some() && behavior_node_parameter(node, ANIMATION_VALUE_PARAMETER_KEY).is_some() {
        return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: ANIMATION_VALUE_PARAMETER_KEY.to_string(),
            expected: "no `value` when `trigger` is declared",
            actual: "declared".to_string(),
        });
    }
    if let Some(value) = behavior_node_parameter(node, ANIMATION_VALUE_PARAMETER_KEY) {
        match value {
            AiBehaviorNodeParameterValue::Bool(_)
            | AiBehaviorNodeParameterValue::Scalar(_)
            | AiBehaviorNodeParameterValue::Vec3(_) => {}
            AiBehaviorNodeParameterValue::Integer(value) if i32::try_from(*value).is_ok() => {}
            _ => {
                return invalid_parameter(
                    tree_id,
                    node,
                    ANIMATION_VALUE_PARAMETER_KEY,
                    "bool, i32 integer, scalar, or vec3",
                    value,
                )
            }
        }
    }
    Ok(())
}

fn validate_script_task(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    has_placeholder_result: bool,
) -> Result<(), AiManagerError> {
    let Some(value) = behavior_node_parameter(node, SCRIPT_CALLBACK_PARAMETER_KEY) else {
        if has_placeholder_result {
            return Ok(());
        }
        return missing_parameter(tree_id, node, SCRIPT_CALLBACK_PARAMETER_KEY, "string");
    };
    let callback = expect_string_parameter(tree_id, node, SCRIPT_CALLBACK_PARAMETER_KEY, value)?;
    ensure_non_empty_parameter(tree_id, node, SCRIPT_CALLBACK_PARAMETER_KEY, callback)?;
    ScriptBehaviorCallbackRef::parse(callback).map_err(|error| {
        AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: SCRIPT_CALLBACK_PARAMETER_KEY.to_string(),
            expected: "a provider-qualified `<package>::<node-id>` callback",
            actual: error.message,
        }
    })?;
    Ok(())
}

fn missing_parameter<T>(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    expected: &'static str,
) -> Result<T, AiManagerError> {
    Err(AiManagerError::InvalidBehaviorNodeParameter {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        key: key.to_string(),
        expected,
        actual: "missing",
    })
}

fn ensure_non_empty_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &str,
) -> Result<(), AiManagerError> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(AiManagerError::InvalidBehaviorNodeParameterValue {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        key: key.to_string(),
        expected: "a non-empty string",
        actual: value.to_string(),
    })
}
