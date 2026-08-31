use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::ai::AI_BEHAVIOR_TREE_FORMAT_VERSION;
use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor, AiManagerError,
};

use super::parameters::{
    parse_parallel_policy, parse_perception_sense, parse_task_result,
    BLACKBOARD_CONDITION_PARAMETER_KEYS, BLACKBOARD_EXISTS_PARAMETER_KEY,
    BLACKBOARD_INVERT_PARAMETER_KEY, BLACKBOARD_KEY_PARAMETER_KEY, DECORATOR_PARAMETER_KEYS,
    NON_NEGATIVE_SCALAR_EXPECTED_VALUE, PARALLEL_FAILURE_POLICY_PARAMETER_KEY,
    PARALLEL_POLICY_EXPECTED_VALUES, PARALLEL_POLICY_PARAMETER_KEYS,
    PARALLEL_SUCCESS_POLICY_PARAMETER_KEY, PERCEPTION_CONDITION_PARAMETER_KEYS,
    PERCEPTION_EXISTS_PARAMETER_KEY, PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    PERCEPTION_MIN_STRENGTH_PARAMETER_KEY, PERCEPTION_SENSE_EXPECTED_VALUES,
    PERCEPTION_SENSE_PARAMETER_KEY, PERCEPTION_SOURCE_PARAMETER_KEY,
    SUBTREE_TARGET_EXPECTED_VALUES, SUBTREE_TARGET_PARAMETER_KEY, TASK_RESULT_EXPECTED_VALUES,
    TASK_RESULT_PARAMETER_KEY,
};

mod integration;
mod runtime_inputs;

#[cfg(test)]
mod topology_index_tests;

pub(super) use runtime_inputs::{
    validate_blackboard_entries, validate_blackboard_schema_descriptor,
    validate_perception_snapshot,
};

pub(super) fn validate_behavior_tree_descriptor(
    descriptor: &AiBehaviorTreeDescriptor,
    registered_tree_ids: &[&str],
) -> Result<(), AiManagerError> {
    validate_behavior_tree_descriptor_inner(descriptor, registered_tree_ids, true)
}

pub(crate) fn validate_behavior_tree_descriptor_for_compile(
    descriptor: &AiBehaviorTreeDescriptor,
) -> Result<(), AiManagerError> {
    validate_behavior_tree_descriptor_inner(descriptor, &[], false)
}

fn validate_behavior_tree_descriptor_inner(
    descriptor: &AiBehaviorTreeDescriptor,
    registered_tree_ids: &[&str],
    require_registered_subtree_target: bool,
) -> Result<(), AiManagerError> {
    if descriptor.format_version != AI_BEHAVIOR_TREE_FORMAT_VERSION {
        return Err(AiManagerError::InvalidBehaviorTreeFormatVersion {
            expected: AI_BEHAVIOR_TREE_FORMAT_VERSION,
            actual: descriptor.format_version,
        });
    }
    ensure_non_empty(&descriptor.id, "behavior_tree.id")?;
    ensure_non_empty(&descriptor.root_node, "behavior_tree.root_node")?;

    let registered_tree_index = (require_registered_subtree_target
        && descriptor
            .nodes
            .iter()
            .any(|node| node.kind == AiBehaviorNodeKind::Subtree))
    .then(|| {
        let mut index = HashSet::with_capacity(registered_tree_ids.len());
        index.extend(registered_tree_ids.iter().copied());
        index
    });
    let mut node_indices = HashMap::with_capacity(descriptor.nodes.len());
    for (node_index, node) in descriptor.nodes.iter().enumerate() {
        ensure_non_empty(&node.id, "behavior_node.id")?;
        if node_indices.insert(node.id.as_str(), node_index).is_some() {
            return Err(AiManagerError::DuplicateId {
                id: node.id.clone(),
            });
        }
        let mut parameter_keys = HashSet::with_capacity(node.parameters.len());
        for parameter in &node.parameters {
            ensure_non_empty(&parameter.key, "behavior_node.parameter")?;
            if !parameter.value.is_finite() {
                return Err(AiManagerError::NonFiniteBehaviorNodeParameter {
                    tree_id: descriptor.id.clone(),
                    node_id: node.id.clone(),
                    key: parameter.key.clone(),
                });
            }
            if !parameter_keys.insert(parameter.key.as_str()) {
                return Err(AiManagerError::DuplicateBehaviorNodeParameter {
                    tree_id: descriptor.id.clone(),
                    node_id: node.id.clone(),
                    key: parameter.key.clone(),
                });
            }
        }
        validate_behavior_node_child_count(&descriptor.id, node)?;
        validate_builtin_behavior_node_parameters(
            &descriptor.id,
            node,
            registered_tree_index.as_ref(),
        )?;
    }

    if !node_indices.contains_key(descriptor.root_node.as_str()) {
        return Err(AiManagerError::MissingRootNode {
            tree_id: descriptor.id.clone(),
            root_node: descriptor.root_node.clone(),
        });
    }

    for node in &descriptor.nodes {
        for child in &node.children {
            ensure_non_empty(child, "behavior_node.child")?;
            if !node_indices.contains_key(child.as_str()) {
                return Err(AiManagerError::MissingChildNode {
                    tree_id: descriptor.id.clone(),
                    node_id: node.id.clone(),
                    child_id: child.clone(),
                });
            }
        }
    }

    validate_behavior_tree_topology(descriptor, &node_indices)?;

    Ok(())
}

fn validate_behavior_node_child_count(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), AiManagerError> {
    match node.kind {
        AiBehaviorNodeKind::Selector
        | AiBehaviorNodeKind::Sequence
        | AiBehaviorNodeKind::Parallel => Ok(()),
        AiBehaviorNodeKind::Decorator => expect_child_count(tree_id, node, "exactly one", 1),
        AiBehaviorNodeKind::Task | AiBehaviorNodeKind::Service | AiBehaviorNodeKind::Subtree => {
            expect_child_count(tree_id, node, "zero", 0)
        }
    }
}

fn expect_child_count(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    expected: &'static str,
    expected_count: usize,
) -> Result<(), AiManagerError> {
    if node.children.len() == expected_count {
        return Ok(());
    }

    Err(AiManagerError::InvalidBehaviorNodeChildCount {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        expected,
        actual: node.children.len(),
    })
}

const VISIT_UNSEEN: u8 = 0;
const VISIT_ACTIVE: u8 = 1;
const VISIT_COMPLETE: u8 = 2;

fn validate_behavior_tree_topology(
    descriptor: &AiBehaviorTreeDescriptor,
    node_indices: &HashMap<&str, usize>,
) -> Result<(), AiManagerError> {
    let Some(&root_index) = node_indices.get(descriptor.root_node.as_str()) else {
        return Err(AiManagerError::MissingRootNode {
            tree_id: descriptor.id.clone(),
            root_node: descriptor.root_node.clone(),
        });
    };
    let mut visit_states = vec![VISIT_UNSEEN; descriptor.nodes.len()];
    visit_behavior_node(descriptor, root_index, node_indices, &mut visit_states)?;
    validate_behavior_node_incoming_edges(descriptor, node_indices, &visit_states)
}

fn visit_behavior_node(
    descriptor: &AiBehaviorTreeDescriptor,
    node_index: usize,
    node_indices: &HashMap<&str, usize>,
    visit_states: &mut [u8],
) -> Result<(), AiManagerError> {
    let node = &descriptor.nodes[node_index];
    match visit_states[node_index] {
        VISIT_COMPLETE => return Ok(()),
        VISIT_ACTIVE => {
            return invalid_behavior_tree_topology(
                &descriptor.id,
                &node.id,
                "node participates in a cycle",
            )
        }
        _ => {}
    }

    visit_states[node_index] = VISIT_ACTIVE;
    for child in &node.children {
        let Some(&child_index) = node_indices.get(child.as_str()) else {
            return Err(AiManagerError::MissingChildNode {
                tree_id: descriptor.id.clone(),
                node_id: node.id.clone(),
                child_id: child.clone(),
            });
        };
        visit_behavior_node(descriptor, child_index, node_indices, visit_states)?;
    }
    visit_states[node_index] = VISIT_COMPLETE;
    Ok(())
}

fn validate_behavior_node_incoming_edges(
    descriptor: &AiBehaviorTreeDescriptor,
    node_indices: &HashMap<&str, usize>,
    visit_states: &[u8],
) -> Result<(), AiManagerError> {
    let mut incoming_edges = vec![0_usize; descriptor.nodes.len()];

    for node in &descriptor.nodes {
        for child in &node.children {
            if let Some(&child_index) = node_indices.get(child.as_str()) {
                incoming_edges[child_index] += 1;
            }
        }
    }

    let root_index = node_indices[descriptor.root_node.as_str()];
    if incoming_edges[root_index] != 0 {
        return invalid_behavior_tree_topology(
            &descriptor.id,
            &descriptor.root_node,
            "root node must not have an incoming edge",
        );
    }

    for (node_index, node) in descriptor.nodes.iter().enumerate() {
        if visit_states[node_index] != VISIT_COMPLETE {
            return invalid_behavior_tree_topology(
                &descriptor.id,
                &node.id,
                "node is not reachable from root",
            );
        }
    }

    for (node_index, node) in descriptor.nodes.iter().enumerate() {
        if node.id != descriptor.root_node && incoming_edges[node_index] != 1 {
            return invalid_behavior_tree_topology(
                &descriptor.id,
                &node.id,
                "node must have exactly one incoming edge",
            );
        }
    }

    Ok(())
}

fn invalid_behavior_tree_topology<T>(
    tree_id: &str,
    node_id: &str,
    reason: &'static str,
) -> Result<T, AiManagerError> {
    Err(AiManagerError::InvalidBehaviorTreeTopology {
        tree_id: tree_id.to_string(),
        node_id: node_id.to_string(),
        reason,
    })
}

fn validate_builtin_behavior_node_parameters(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    registered_tree_index: Option<&HashSet<&str>>,
) -> Result<(), AiManagerError> {
    validate_builtin_behavior_node_parameter_owners(tree_id, node)?;
    validate_subtree_target_parameter(tree_id, node, registered_tree_index)?;

    if let Some(value) = behavior_node_parameter(node, TASK_RESULT_PARAMETER_KEY) {
        let result = expect_string_parameter(tree_id, node, TASK_RESULT_PARAMETER_KEY, value)?;
        if parse_task_result(result).is_none() {
            return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
                tree_id: tree_id.to_string(),
                node_id: node.id.clone(),
                key: TASK_RESULT_PARAMETER_KEY.to_string(),
                expected: TASK_RESULT_EXPECTED_VALUES,
                actual: result.to_string(),
            });
        }
    }
    validate_parallel_policy_parameter(tree_id, node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY)?;
    validate_parallel_policy_parameter(tree_id, node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY)?;
    validate_standard_node_parameters(tree_id, node)?;
    integration::validate_integration_node_parameters(tree_id, node)?;

    let has_blackboard_condition =
        has_any_behavior_node_parameter(node, BLACKBOARD_CONDITION_PARAMETER_KEYS);
    let has_perception_condition =
        has_any_behavior_node_parameter(node, PERCEPTION_CONDITION_PARAMETER_KEYS);
    let has_invert = behavior_node_parameter(node, BLACKBOARD_INVERT_PARAMETER_KEY).is_some();
    if let Some(value) = behavior_node_parameter(node, BLACKBOARD_KEY_PARAMETER_KEY) {
        expect_string_parameter(tree_id, node, BLACKBOARD_KEY_PARAMETER_KEY, value)?;
    } else if has_blackboard_condition || (has_invert && !has_perception_condition) {
        return Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: BLACKBOARD_KEY_PARAMETER_KEY.to_string(),
            expected: "string",
            actual: "missing",
        });
    }

    if let Some(value) = behavior_node_parameter(node, BLACKBOARD_EXISTS_PARAMETER_KEY) {
        expect_bool_parameter(tree_id, node, BLACKBOARD_EXISTS_PARAMETER_KEY, value)?;
    }
    if let Some(value) = behavior_node_parameter(node, BLACKBOARD_INVERT_PARAMETER_KEY) {
        expect_bool_parameter(tree_id, node, BLACKBOARD_INVERT_PARAMETER_KEY, value)?;
    }
    validate_perception_condition_parameters(tree_id, node)?;
    if let Some(value) = behavior_node_parameter(node, "equals_bool") {
        expect_bool_parameter(tree_id, node, "equals_bool", value)?;
    }
    if let Some(value) = behavior_node_parameter(node, "equals_string") {
        expect_string_parameter(tree_id, node, "equals_string", value)?;
    }
    if let Some(value) = behavior_node_parameter(node, "equals_integer") {
        expect_integer_parameter(tree_id, node, "equals_integer", value)?;
    }
    if let Some(value) = behavior_node_parameter(node, "equals_scalar") {
        expect_scalar_parameter(tree_id, node, "equals_scalar", value)?;
    }
    if let Some(value) = behavior_node_parameter(node, "equals_vec3") {
        expect_vec3_parameter(tree_id, node, "equals_vec3", value)?;
    }
    if let Some(value) = behavior_node_parameter(node, "equals_entity") {
        expect_entity_parameter(tree_id, node, "equals_entity", value)?;
    }
    validate_integer_comparison_parameter(tree_id, node, "greater_than_integer")?;
    validate_integer_comparison_parameter(tree_id, node, "greater_or_equal_integer")?;
    validate_integer_comparison_parameter(tree_id, node, "less_than_integer")?;
    validate_integer_comparison_parameter(tree_id, node, "less_or_equal_integer")?;
    validate_scalar_comparison_parameter(tree_id, node, "greater_than_scalar")?;
    validate_scalar_comparison_parameter(tree_id, node, "greater_or_equal_scalar")?;
    validate_scalar_comparison_parameter(tree_id, node, "less_than_scalar")?;
    validate_scalar_comparison_parameter(tree_id, node, "less_or_equal_scalar")?;

    Ok(())
}

fn validate_standard_node_parameters(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), AiManagerError> {
    for key in ["duration_seconds", "cooldown_seconds", "time_limit_seconds"] {
        validate_non_negative_scalar_parameter(tree_id, node, key)?;
    }
    for parameter in &node.parameters {
        if parameter.key.starts_with("weight.") || parameter.key.starts_with("weight_") {
            validate_non_negative_scalar_parameter(tree_id, node, &parameter.key)?;
        }
    }
    if let Some(value) = behavior_node_parameter(node, "count") {
        let AiBehaviorNodeParameterValue::Integer(count) = value else {
            return invalid_parameter(tree_id, node, "count", "integer", value);
        };
        if *count <= 0 {
            return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
                tree_id: tree_id.to_string(),
                node_id: node.id.clone(),
                key: "count".to_string(),
                expected: "a positive integer",
                actual: count.to_string(),
            });
        }
    }
    if let Some(value) = behavior_node_parameter(node, "infinite") {
        expect_bool_parameter(tree_id, node, "infinite", value)?;
    }
    for key in ["forced_result", "service_result"] {
        let Some(value) = behavior_node_parameter(node, key) else {
            continue;
        };
        let result = expect_string_parameter(tree_id, node, key, value)?;
        if parse_task_result(result).is_none() {
            return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
                tree_id: tree_id.to_string(),
                node_id: node.id.clone(),
                key: key.to_string(),
                expected: TASK_RESULT_EXPECTED_VALUES,
                actual: result.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_perception_condition_parameters(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), AiManagerError> {
    if let Some(value) = behavior_node_parameter(node, PERCEPTION_SENSE_PARAMETER_KEY) {
        let sense = expect_string_parameter(tree_id, node, PERCEPTION_SENSE_PARAMETER_KEY, value)?;
        if parse_perception_sense(sense).is_none() {
            return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
                tree_id: tree_id.to_string(),
                node_id: node.id.clone(),
                key: PERCEPTION_SENSE_PARAMETER_KEY.to_string(),
                expected: PERCEPTION_SENSE_EXPECTED_VALUES,
                actual: sense.to_string(),
            });
        }
    }
    if let Some(value) = behavior_node_parameter(node, PERCEPTION_SOURCE_PARAMETER_KEY) {
        expect_entity_parameter(tree_id, node, PERCEPTION_SOURCE_PARAMETER_KEY, value)?;
    }
    validate_non_negative_scalar_parameter(tree_id, node, PERCEPTION_MIN_STRENGTH_PARAMETER_KEY)?;
    validate_non_negative_scalar_parameter(
        tree_id,
        node,
        PERCEPTION_MAX_AGE_SECONDS_PARAMETER_KEY,
    )?;
    if let Some(value) = behavior_node_parameter(node, PERCEPTION_EXISTS_PARAMETER_KEY) {
        expect_bool_parameter(tree_id, node, PERCEPTION_EXISTS_PARAMETER_KEY, value)?;
    }

    Ok(())
}

fn validate_builtin_behavior_node_parameter_owners(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), AiManagerError> {
    for parameter in &node.parameters {
        let Some(expected) = expected_builtin_parameter_owner(parameter.key.as_str(), node.kind)
        else {
            continue;
        };
        return Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: parameter.key.clone(),
            expected,
        });
    }

    Ok(())
}

fn expected_builtin_parameter_owner(
    key: &str,
    node_kind: AiBehaviorNodeKind,
) -> Option<&'static str> {
    if key == TASK_RESULT_PARAMETER_KEY && node_kind != AiBehaviorNodeKind::Task {
        return Some("`task` nodes");
    }
    if PARALLEL_POLICY_PARAMETER_KEYS.contains(&key) && node_kind != AiBehaviorNodeKind::Parallel {
        return Some("`parallel` nodes");
    }
    if DECORATOR_PARAMETER_KEYS.contains(&key) && node_kind != AiBehaviorNodeKind::Decorator {
        return Some("`decorator` nodes");
    }
    if key == SUBTREE_TARGET_PARAMETER_KEY && node_kind != AiBehaviorNodeKind::Subtree {
        return Some("`subtree` nodes");
    }
    None
}

fn validate_subtree_target_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    registered_tree_index: Option<&HashSet<&str>>,
) -> Result<(), AiManagerError> {
    if node.kind != AiBehaviorNodeKind::Subtree {
        return Ok(());
    }

    let Some(value) = behavior_node_parameter(node, SUBTREE_TARGET_PARAMETER_KEY) else {
        return Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: SUBTREE_TARGET_PARAMETER_KEY.to_string(),
            expected: "string",
            actual: "missing",
        });
    };
    let target_tree = expect_string_parameter(tree_id, node, SUBTREE_TARGET_PARAMETER_KEY, value)?;
    if target_tree.trim().is_empty() {
        return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: SUBTREE_TARGET_PARAMETER_KEY.to_string(),
            expected: SUBTREE_TARGET_EXPECTED_VALUES,
            actual: target_tree.to_string(),
        });
    }
    if target_tree == tree_id {
        return invalid_subtree_target(tree_id, node, target_tree, "subtree cannot target itself");
    }
    if registered_tree_index.is_some_and(|index| !index.contains(target_tree)) {
        return invalid_subtree_target(
            tree_id,
            node,
            target_tree,
            "target behavior tree is not registered",
        );
    }

    Ok(())
}

fn invalid_subtree_target<T>(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    target_tree: &str,
    reason: &'static str,
) -> Result<T, AiManagerError> {
    Err(AiManagerError::InvalidBehaviorSubtreeTarget {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        target_tree: target_tree.to_string(),
        reason,
    })
}

fn validate_integer_comparison_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
) -> Result<(), AiManagerError> {
    if let Some(value) = behavior_node_parameter(node, key) {
        expect_integer_parameter(tree_id, node, key, value)?;
    }
    Ok(())
}

fn validate_scalar_comparison_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
) -> Result<(), AiManagerError> {
    if let Some(value) = behavior_node_parameter(node, key) {
        expect_scalar_parameter(tree_id, node, key, value)?;
    }
    Ok(())
}

fn validate_non_negative_scalar_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
) -> Result<(), AiManagerError> {
    let Some(value) = behavior_node_parameter(node, key) else {
        return Ok(());
    };
    let scalar = expect_scalar_parameter(tree_id, node, key, value)?;
    if scalar < 0.0 {
        return Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: tree_id.to_string(),
            node_id: node.id.clone(),
            key: key.to_string(),
            expected: NON_NEGATIVE_SCALAR_EXPECTED_VALUE,
            actual: scalar.to_string(),
        });
    }
    Ok(())
}

fn validate_parallel_policy_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
) -> Result<(), AiManagerError> {
    let Some(value) = behavior_node_parameter(node, key) else {
        return Ok(());
    };
    let policy = expect_string_parameter(tree_id, node, key, value)?;
    if parse_parallel_policy(policy).is_some() {
        return Ok(());
    }

    Err(AiManagerError::InvalidBehaviorNodeParameterValue {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        key: key.to_string(),
        expected: PARALLEL_POLICY_EXPECTED_VALUES,
        actual: policy.to_string(),
    })
}

fn behavior_node_parameter<'a>(
    node: &'a AiBehaviorNodeDescriptor,
    key: &str,
) -> Option<&'a AiBehaviorNodeParameterValue> {
    node.parameters
        .iter()
        .find(|parameter| parameter.key == key)
        .map(|parameter| &parameter.value)
}

fn has_any_behavior_node_parameter(node: &AiBehaviorNodeDescriptor, keys: &[&str]) -> bool {
    node.parameters
        .iter()
        .any(|parameter| keys.contains(&parameter.key.as_str()))
}

fn expect_string_parameter<'a>(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &'a AiBehaviorNodeParameterValue,
) -> Result<&'a str, AiManagerError> {
    if let AiBehaviorNodeParameterValue::String(value) = value {
        return Ok(value);
    }
    invalid_parameter(tree_id, node, key, "string", value)
}

fn expect_bool_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<(), AiManagerError> {
    if !matches!(value, AiBehaviorNodeParameterValue::Bool(_)) {
        return invalid_parameter(tree_id, node, key, "bool", value);
    }
    Ok(())
}

fn expect_integer_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<(), AiManagerError> {
    if !matches!(value, AiBehaviorNodeParameterValue::Integer(_)) {
        return invalid_parameter(tree_id, node, key, "integer", value);
    }
    Ok(())
}

fn expect_scalar_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<f32, AiManagerError> {
    if let AiBehaviorNodeParameterValue::Scalar(value) = value {
        return Ok(*value);
    }
    invalid_parameter(tree_id, node, key, "scalar", value)
}

fn expect_vec3_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<(), AiManagerError> {
    if !matches!(value, AiBehaviorNodeParameterValue::Vec3(_)) {
        return invalid_parameter(tree_id, node, key, "vec3", value);
    }
    Ok(())
}

fn expect_entity_parameter(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<(), AiManagerError> {
    if !matches!(value, AiBehaviorNodeParameterValue::Entity(_)) {
        return invalid_parameter(tree_id, node, key, "entity", value);
    }
    Ok(())
}

fn invalid_parameter<T>(
    tree_id: &str,
    node: &AiBehaviorNodeDescriptor,
    key: &str,
    expected: &'static str,
    value: &AiBehaviorNodeParameterValue,
) -> Result<T, AiManagerError> {
    Err(AiManagerError::InvalidBehaviorNodeParameter {
        tree_id: tree_id.to_string(),
        node_id: node.id.clone(),
        key: key.to_string(),
        expected,
        actual: value.value_type(),
    })
}

fn ensure_non_empty(value: &str, field: &'static str) -> Result<(), AiManagerError> {
    if value.trim().is_empty() {
        return Err(AiManagerError::EmptyId { field });
    }
    Ok(())
}
