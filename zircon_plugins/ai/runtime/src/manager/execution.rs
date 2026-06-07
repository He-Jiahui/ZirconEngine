use std::collections::HashMap;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardValue, AiDecisionStatus,
    AiPerceptionSnapshot, AiPerceptionStimulus,
};

use super::parameters::{
    parse_parallel_policy, parse_perception_sense, parse_task_result, ParallelPolicy,
    BLACKBOARD_EXISTS_PARAMETER_KEY, BLACKBOARD_INVERT_PARAMETER_KEY, BLACKBOARD_KEY_PARAMETER_KEY,
    DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS, PARALLEL_FAILURE_POLICY_PARAMETER_KEY,
    PARALLEL_SUCCESS_POLICY_PARAMETER_KEY, PERCEPTION_CONDITION_PARAMETER_KEYS,
    PERCEPTION_EXISTS_PARAMETER_KEY, PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    PERCEPTION_MIN_STRENGTH_PARAMETER_KEY, PERCEPTION_SENSE_PARAMETER_KEY,
    PERCEPTION_SOURCE_PARAMETER_KEY, SUBTREE_TARGET_PARAMETER_KEY, TASK_RESULT_PARAMETER_KEY,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BehaviorTreeExecution {
    pub(super) status: AiDecisionStatus,
    pub(super) active_node: Option<String>,
    pub(super) diagnostic: Option<String>,
}

struct BehaviorTreeExecutionContext<'a> {
    blackboard: &'a [AiBlackboardEntry],
    perception: Option<&'a AiPerceptionSnapshot>,
}

pub(super) fn evaluate_behavior_tree(
    descriptor: &AiBehaviorTreeDescriptor,
    registered_trees: &[AiBehaviorTreeDescriptor],
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
) -> BehaviorTreeExecution {
    let tree_descriptors = registered_trees
        .iter()
        .map(|tree| (tree.id.as_str(), tree))
        .collect::<HashMap<_, _>>();
    let context = BehaviorTreeExecutionContext {
        blackboard,
        perception,
    };
    evaluate_behavior_tree_with_stack(descriptor, &tree_descriptors, &context, &mut Vec::new())
}

fn evaluate_behavior_tree_with_stack(
    descriptor: &AiBehaviorTreeDescriptor,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let nodes = descriptor
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    tree_stack.push(descriptor.id.clone());
    let result = evaluate_node(
        &descriptor.root_node,
        &nodes,
        tree_descriptors,
        context,
        tree_stack,
    );
    tree_stack.pop();
    result
}

fn evaluate_node(
    node_id: &str,
    nodes: &HashMap<&str, &AiBehaviorNodeDescriptor>,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let Some(node) = nodes.get(node_id).copied() else {
        return blocked(node_id, "references a missing behavior-tree node");
    };

    match node.kind {
        AiBehaviorNodeKind::Selector => {
            evaluate_selector(node, nodes, tree_descriptors, context, tree_stack)
        }
        AiBehaviorNodeKind::Sequence => {
            evaluate_sequence(node, nodes, tree_descriptors, context, tree_stack)
        }
        AiBehaviorNodeKind::Parallel => {
            evaluate_parallel(node, nodes, tree_descriptors, context, tree_stack)
        }
        AiBehaviorNodeKind::Decorator => {
            evaluate_decorator(node, nodes, tree_descriptors, context, tree_stack)
        }
        AiBehaviorNodeKind::Task => evaluate_task(node),
        AiBehaviorNodeKind::Subtree => {
            evaluate_subtree(node, tree_descriptors, context, tree_stack)
        }
        AiBehaviorNodeKind::Service => blocked(
            &node.id,
            "requires a specialized behavior-tree executor that is not registered yet",
        ),
    }
}

fn evaluate_selector(
    node: &AiBehaviorNodeDescriptor,
    nodes: &HashMap<&str, &AiBehaviorNodeDescriptor>,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let mut last_failed = None;
    for child in &node.children {
        let result = evaluate_node(child, nodes, tree_descriptors, context, tree_stack);
        if result.status != AiDecisionStatus::Failed {
            return result;
        }
        last_failed = Some(result);
    }

    last_failed.unwrap_or_else(|| BehaviorTreeExecution {
        status: AiDecisionStatus::Failed,
        active_node: Some(node.id.clone()),
        diagnostic: None,
    })
}

fn evaluate_sequence(
    node: &AiBehaviorNodeDescriptor,
    nodes: &HashMap<&str, &AiBehaviorNodeDescriptor>,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let mut last_succeeded = None;
    for child in &node.children {
        let result = evaluate_node(child, nodes, tree_descriptors, context, tree_stack);
        if result.status != AiDecisionStatus::Succeeded {
            return result;
        }
        last_succeeded = Some(result);
    }

    last_succeeded.unwrap_or_else(|| BehaviorTreeExecution {
        status: AiDecisionStatus::Succeeded,
        active_node: Some(node.id.clone()),
        diagnostic: None,
    })
}

fn evaluate_parallel(
    node: &AiBehaviorNodeDescriptor,
    nodes: &HashMap<&str, &AiBehaviorNodeDescriptor>,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    // This is a deterministic descriptor fold, not a thread/task scheduler. A later executor can
    // reuse the same policies when latent tasks and services become runtime state.
    let success_policy =
        parallel_policy(node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::All);
    let failure_policy =
        parallel_policy(node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::Any);
    let child_results = node
        .children
        .iter()
        .map(|child| evaluate_node(child, nodes, tree_descriptors, context, tree_stack))
        .collect::<Vec<_>>();

    if let Some(blocked) = first_status(&child_results, AiDecisionStatus::Blocked) {
        return blocked.clone();
    }
    if parallel_policy_matches(&child_results, AiDecisionStatus::Succeeded, success_policy) {
        return selected_parallel_result(&child_results, AiDecisionStatus::Succeeded)
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Succeeded));
    }
    if parallel_policy_matches(&child_results, AiDecisionStatus::Failed, failure_policy) {
        return selected_parallel_result(&child_results, AiDecisionStatus::Failed)
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Failed));
    }
    if let Some(running) = first_status(&child_results, AiDecisionStatus::Running) {
        return running.clone();
    }
    if let Some(idle) = first_status(&child_results, AiDecisionStatus::Idle) {
        return idle.clone();
    }

    node_result(node, AiDecisionStatus::Running)
}

fn evaluate_decorator(
    node: &AiBehaviorNodeDescriptor,
    nodes: &HashMap<&str, &AiBehaviorNodeDescriptor>,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    if !decorator_condition_passes(node, context) {
        return BehaviorTreeExecution {
            status: AiDecisionStatus::Failed,
            active_node: Some(node.id.clone()),
            diagnostic: None,
        };
    }

    let Some(child) = node.children.first() else {
        return BehaviorTreeExecution {
            status: AiDecisionStatus::Succeeded,
            active_node: Some(node.id.clone()),
            diagnostic: None,
        };
    };
    evaluate_node(child, nodes, tree_descriptors, context, tree_stack)
}

fn evaluate_task(node: &AiBehaviorNodeDescriptor) -> BehaviorTreeExecution {
    let status = parameter(node, TASK_RESULT_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .and_then(parse_task_result)
        .unwrap_or(AiDecisionStatus::Running);

    BehaviorTreeExecution {
        status,
        active_node: Some(node.id.clone()),
        diagnostic: None,
    }
}

fn evaluate_subtree(
    node: &AiBehaviorNodeDescriptor,
    tree_descriptors: &HashMap<&str, &AiBehaviorTreeDescriptor>,
    context: &BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let Some(target_tree_id) = parameter(node, SUBTREE_TARGET_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    else {
        return blocked(
            &node.id,
            "subtree node does not declare a target behavior tree",
        );
    };
    let Some(target_tree) = tree_descriptors.get(target_tree_id).copied() else {
        return blocked(
            &node.id,
            "subtree node references an unregistered behavior tree",
        );
    };
    if tree_stack
        .iter()
        .any(|tree_id| tree_id.as_str() == target_tree_id)
    {
        return blocked(
            &node.id,
            "subtree node would re-enter an active behavior tree",
        );
    }

    let mut result =
        evaluate_behavior_tree_with_stack(target_tree, tree_descriptors, context, tree_stack);
    if let Some(active_node) = result.active_node.take() {
        result.active_node = Some(format!("{target_tree_id}::{active_node}"));
    }
    result
}

fn decorator_condition_passes(
    node: &AiBehaviorNodeDescriptor,
    context: &BehaviorTreeExecutionContext<'_>,
) -> bool {
    let raw_condition_passes = raw_blackboard_condition_passes(node, context.blackboard)
        && raw_perception_condition_passes(node, context.perception);
    if parameter(node, BLACKBOARD_INVERT_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
        .unwrap_or(false)
    {
        !raw_condition_passes
    } else {
        raw_condition_passes
    }
}

fn raw_blackboard_condition_passes(
    node: &AiBehaviorNodeDescriptor,
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
    node: &AiBehaviorNodeDescriptor,
    perception: Option<&AiPerceptionSnapshot>,
) -> bool {
    if !has_perception_condition(node) {
        return true;
    }

    let matching_stimulus_exists = perception
        .map(|snapshot| {
            snapshot
                .stimuli
                .iter()
                .any(|stimulus| perception_stimulus_matches(node, stimulus))
        })
        .unwrap_or(false);
    let expected_exists = parameter(node, PERCEPTION_EXISTS_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_bool)
        .unwrap_or(true);

    matching_stimulus_exists == expected_exists
}

fn has_perception_condition(node: &AiBehaviorNodeDescriptor) -> bool {
    node.parameters
        .iter()
        .any(|parameter| PERCEPTION_CONDITION_PARAMETER_KEYS.contains(&parameter.key.as_str()))
}

fn perception_stimulus_matches(
    node: &AiBehaviorNodeDescriptor,
    stimulus: &AiPerceptionStimulus,
) -> bool {
    if let Some(expected_sense) = parameter(node, PERCEPTION_SENSE_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .and_then(parse_perception_sense)
    {
        if stimulus.sense != expected_sense {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Entity(expected_source)) =
        parameter(node, PERCEPTION_SOURCE_PARAMETER_KEY)
    {
        if stimulus.source != *expected_source {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(min_strength)) =
        parameter(node, PERCEPTION_MIN_STRENGTH_PARAMETER_KEY)
    {
        if stimulus.strength < *min_strength {
            return false;
        }
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(max_age_seconds)) =
        parameter(node, PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY)
    {
        if stimulus.age_seconds > *max_age_seconds {
            return false;
        }
    }

    true
}

fn has_value_comparison(node: &AiBehaviorNodeDescriptor) -> bool {
    node.parameters.iter().any(|parameter| {
        DECORATOR_VALUE_COMPARISON_PARAMETER_KEYS.contains(&parameter.key.as_str())
    })
}

fn value_comparison_passes(node: &AiBehaviorNodeDescriptor, value: &AiBlackboardValue) -> bool {
    let mut compared = false;
    let mut passed = true;

    if let Some(expected) =
        parameter(node, "equals_bool").and_then(AiBehaviorNodeParameterValue::as_bool)
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Bool(actual) if *actual == expected);
    }
    if let Some(expected) =
        parameter(node, "equals_string").and_then(AiBehaviorNodeParameterValue::as_string)
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::String(actual) if actual == expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Integer(expected)) = parameter(node, "equals_integer")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Integer(actual) if actual == expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(expected)) = parameter(node, "equals_scalar") {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Scalar(actual) if actual == expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Vec3(expected)) = parameter(node, "equals_vec3") {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Vec3(actual) if actual == expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Entity(expected)) = parameter(node, "equals_entity") {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Entity(actual) if actual == expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Integer(expected)) =
        parameter(node, "greater_than_integer")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Integer(actual) if *actual > *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Integer(expected)) =
        parameter(node, "greater_or_equal_integer")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Integer(actual) if *actual >= *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Integer(expected)) =
        parameter(node, "less_than_integer")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Integer(actual) if *actual < *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Integer(expected)) =
        parameter(node, "less_or_equal_integer")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Integer(actual) if *actual <= *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(expected)) =
        parameter(node, "greater_than_scalar")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Scalar(actual) if *actual > *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(expected)) =
        parameter(node, "greater_or_equal_scalar")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Scalar(actual) if *actual >= *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(expected)) =
        parameter(node, "less_than_scalar")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Scalar(actual) if *actual < *expected);
    }
    if let Some(AiBehaviorNodeParameterValue::Scalar(expected)) =
        parameter(node, "less_or_equal_scalar")
    {
        compared = true;
        passed &= matches!(value, AiBlackboardValue::Scalar(actual) if *actual <= *expected);
    }

    !compared || passed
}

fn parameter<'a>(
    node: &'a AiBehaviorNodeDescriptor,
    key: &str,
) -> Option<&'a AiBehaviorNodeParameterValue> {
    node.parameters
        .iter()
        .find(|parameter| parameter.key == key)
        .map(|parameter| &parameter.value)
}

fn parallel_policy(node: &AiBehaviorNodeDescriptor, key: &str) -> Option<ParallelPolicy> {
    parameter(node, key)?
        .as_string()
        .and_then(parse_parallel_policy)
}

fn parallel_policy_matches(
    child_results: &[BehaviorTreeExecution],
    status: AiDecisionStatus,
    policy: ParallelPolicy,
) -> bool {
    match policy {
        ParallelPolicy::All => {
            !child_results.is_empty() && child_results.iter().all(|result| result.status == status)
        }
        ParallelPolicy::Any => child_results.iter().any(|result| result.status == status),
    }
}

fn selected_parallel_result(
    child_results: &[BehaviorTreeExecution],
    status: AiDecisionStatus,
) -> Option<BehaviorTreeExecution> {
    child_results
        .iter()
        .rev()
        .find(|result| result.status == status)
        .cloned()
}

fn first_status(
    child_results: &[BehaviorTreeExecution],
    status: AiDecisionStatus,
) -> Option<&BehaviorTreeExecution> {
    child_results.iter().find(|result| result.status == status)
}

fn node_result(node: &AiBehaviorNodeDescriptor, status: AiDecisionStatus) -> BehaviorTreeExecution {
    BehaviorTreeExecution {
        status,
        active_node: Some(node.id.clone()),
        diagnostic: None,
    }
}

fn blocked(node_id: &str, reason: &'static str) -> BehaviorTreeExecution {
    BehaviorTreeExecution {
        status: AiDecisionStatus::Blocked,
        active_node: Some(node_id.to_string()),
        diagnostic: Some(format!("AI behavior node `{node_id}` {reason}")),
    }
}
