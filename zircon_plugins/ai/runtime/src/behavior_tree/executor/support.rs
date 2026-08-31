use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameterValue, AiDecisionStatus, AiManagerError,
};

use crate::blackboard::BlackboardLayout;
use crate::manager::parameters::{parse_parallel_policy, ParallelPolicy};

use super::{
    BehaviorTreeExecution, BehaviorTreeInstanceState, CompiledBehaviorNode, CompiledBehaviorTree,
};

#[cfg(test)]
#[path = "support/weighted_random_tests.rs"]
mod weighted_random_tests;

#[cfg(test)]
#[path = "support/observer_binding_tests.rs"]
mod observer_binding_tests;

pub(super) fn bind_reachable_observers(
    instance: &mut BehaviorTreeInstanceState,
    root: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
    layout: &BlackboardLayout,
) -> Result<(), AiManagerError> {
    if instance.observer_binding_root.as_deref() == Some(root.id())
        && instance.observer_binding_schema.as_deref() == Some(layout.schema_id())
    {
        return Ok(());
    }
    for tree in super::super::reachable_behavior_trees(root, registered_trees) {
        instance.bind_observers(tree, layout)?;
    }
    instance.observer_binding_root = Some(root.id().to_string());
    instance.observer_binding_schema = Some(layout.schema_id().to_string());
    Ok(())
}

pub(super) fn weighted_random_child(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    children: &[u32],
    tick: u64,
) -> u32 {
    use std::hash::{Hash, Hasher};

    let weights = children
        .iter()
        .enumerate()
        .map(|(position, child)| {
            borrowed_child_weight(node, tree.node(*child as usize).id(), position)
        })
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return children[0];
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tree.id().hash(&mut hasher);
    node.id().hash(&mut hasher);
    tick.hash(&mut hasher);
    let mut sample = (hasher.finish() as f64 / u64::MAX as f64) as f32 * total;
    for (child, weight) in children.iter().zip(weights) {
        if sample < weight {
            return *child;
        }
        sample -= weight;
    }
    children[children.len() - 1]
}

fn borrowed_child_weight(node: &CompiledBehaviorNode, child_id: &str, position: usize) -> f32 {
    let id_parameter = node
        .parameters()
        .iter()
        .find(|parameter| parameter.key.strip_prefix("weight.") == Some(child_id));
    if let Some(AiBehaviorNodeParameterValue::Scalar(value)) =
        id_parameter.map(|parameter| &parameter.value)
    {
        return value.max(0.0);
    }

    let position_parameter = node.parameters().iter().find(|parameter| {
        parameter
            .key
            .strip_prefix("weight_")
            .is_some_and(|suffix| canonical_index_suffix_matches(suffix, position))
    });
    match position_parameter.map(|parameter| &parameter.value) {
        Some(AiBehaviorNodeParameterValue::Scalar(value)) => value.max(0.0),
        _ => 1.0,
    }
}

fn canonical_index_suffix_matches(suffix: &str, index: usize) -> bool {
    let bytes = suffix.as_bytes();
    if index == 0 {
        return bytes == b"0";
    }
    let mut remaining = index;
    let mut cursor = bytes.len();
    while remaining != 0 {
        if cursor == 0 {
            return false;
        }
        cursor -= 1;
        if bytes[cursor] != b'0' + (remaining % 10) as u8 {
            return false;
        }
        remaining /= 10;
    }
    cursor == 0
}

pub(super) fn parameter<'a>(
    node: &'a CompiledBehaviorNode,
    key: &str,
) -> Option<&'a AiBehaviorNodeParameterValue> {
    node.parameters()
        .iter()
        .find(|parameter| parameter.key == key)
        .map(|parameter| &parameter.value)
}

pub(super) fn scalar_parameter(node: &CompiledBehaviorNode, keys: &[&str]) -> Option<f32> {
    keys.iter().find_map(|key| match parameter(node, key) {
        Some(AiBehaviorNodeParameterValue::Scalar(value)) => Some(*value),
        _ => None,
    })
}

pub(super) fn integer_parameter(node: &CompiledBehaviorNode, key: &str) -> Option<i64> {
    match parameter(node, key) {
        Some(AiBehaviorNodeParameterValue::Integer(value)) => Some(*value),
        _ => None,
    }
}

pub(super) fn bool_parameter(node: &CompiledBehaviorNode, key: &str) -> Option<bool> {
    parameter(node, key).and_then(AiBehaviorNodeParameterValue::as_bool)
}

pub(super) fn is_terminal(status: &AiDecisionStatus) -> bool {
    matches!(
        status,
        AiDecisionStatus::Succeeded | AiDecisionStatus::Failed | AiDecisionStatus::Blocked
    )
}

pub(super) fn parallel_policy(node: &CompiledBehaviorNode, key: &str) -> Option<ParallelPolicy> {
    parameter(node, key)?
        .as_string()
        .and_then(parse_parallel_policy)
}

pub(super) fn parallel_policy_matches(
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

pub(super) fn selected_parallel_result(
    child_results: &[BehaviorTreeExecution],
    status: AiDecisionStatus,
) -> Option<BehaviorTreeExecution> {
    child_results
        .iter()
        .rev()
        .find(|result| result.status == status)
        .cloned()
}

pub(super) fn first_status(
    child_results: &[BehaviorTreeExecution],
    status: AiDecisionStatus,
) -> Option<&BehaviorTreeExecution> {
    child_results.iter().find(|result| result.status == status)
}

pub(super) fn node_result(
    node: &CompiledBehaviorNode,
    status: AiDecisionStatus,
) -> BehaviorTreeExecution {
    BehaviorTreeExecution {
        status,
        active_node: Some(node.id().to_string()),
        diagnostic: None,
    }
}

pub(super) fn blocked(node_id: &str, reason: &'static str) -> BehaviorTreeExecution {
    BehaviorTreeExecution {
        status: AiDecisionStatus::Blocked,
        active_node: Some(node_id.to_string()),
        diagnostic: Some(format!("AI behavior node `{node_id}` {reason}")),
    }
}
