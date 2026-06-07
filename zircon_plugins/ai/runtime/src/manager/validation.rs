use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiManagerError,
    AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::EntityId;

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

pub(super) fn validate_behavior_tree_descriptor(
    descriptor: &AiBehaviorTreeDescriptor,
    registered_tree_ids: &[&str],
) -> Result<(), AiManagerError> {
    ensure_non_empty(&descriptor.id, "behavior_tree.id")?;
    ensure_non_empty(&descriptor.root_node, "behavior_tree.root_node")?;

    let mut node_ids = HashSet::new();
    for node in &descriptor.nodes {
        ensure_non_empty(&node.id, "behavior_node.id")?;
        if !node_ids.insert(node.id.as_str()) {
            return Err(AiManagerError::DuplicateId {
                id: node.id.clone(),
            });
        }
        let mut parameter_keys = HashSet::new();
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
        validate_builtin_behavior_node_parameters(&descriptor.id, node, registered_tree_ids)?;
    }

    if !node_ids.contains(descriptor.root_node.as_str()) {
        return Err(AiManagerError::MissingRootNode {
            tree_id: descriptor.id.clone(),
            root_node: descriptor.root_node.clone(),
        });
    }

    for node in &descriptor.nodes {
        for child in &node.children {
            ensure_non_empty(child, "behavior_node.child")?;
            if !node_ids.contains(child.as_str()) {
                return Err(AiManagerError::MissingChildNode {
                    tree_id: descriptor.id.clone(),
                    node_id: node.id.clone(),
                    child_id: child.clone(),
                });
            }
        }
    }

    validate_behavior_tree_topology(descriptor)?;

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

fn validate_behavior_tree_topology(
    descriptor: &AiBehaviorTreeDescriptor,
) -> Result<(), AiManagerError> {
    let nodes = descriptor
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();

    visit_behavior_node(
        &descriptor.id,
        descriptor.root_node.as_str(),
        &nodes,
        &mut visiting,
        &mut visited,
    )?;
    validate_behavior_node_incoming_edges(descriptor, &visited)
}

fn visit_behavior_node<'a>(
    tree_id: &str,
    node_id: &'a str,
    nodes: &HashMap<&'a str, &'a AiBehaviorNodeDescriptor>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) -> Result<(), AiManagerError> {
    if visited.contains(node_id) {
        return Ok(());
    }
    if !visiting.insert(node_id) {
        return invalid_behavior_tree_topology(tree_id, node_id, "node participates in a cycle");
    }

    if let Some(node) = nodes.get(node_id).copied() {
        for child in &node.children {
            let (child_id, _) = nodes
                .get_key_value(child.as_str())
                .expect("child nodes are validated before topology traversal");
            visit_behavior_node(tree_id, *child_id, nodes, visiting, visited)?;
        }
    }

    visiting.remove(node_id);
    visited.insert(node_id);
    Ok(())
}

fn validate_behavior_node_incoming_edges(
    descriptor: &AiBehaviorTreeDescriptor,
    visited: &HashSet<&str>,
) -> Result<(), AiManagerError> {
    let mut incoming_edges = descriptor
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), 0_usize))
        .collect::<HashMap<_, _>>();

    for node in &descriptor.nodes {
        for child in &node.children {
            if let Some(count) = incoming_edges.get_mut(child.as_str()) {
                *count += 1;
            }
        }
    }

    for node in &descriptor.nodes {
        let incoming_count = incoming_edges.get(node.id.as_str()).copied().unwrap_or(0);
        if node.id == descriptor.root_node {
            if incoming_count != 0 {
                return invalid_behavior_tree_topology(
                    &descriptor.id,
                    &node.id,
                    "root node must not have an incoming edge",
                );
            }
        }
    }

    for node in &descriptor.nodes {
        if !visited.contains(node.id.as_str()) {
            return invalid_behavior_tree_topology(
                &descriptor.id,
                &node.id,
                "node is not reachable from root",
            );
        }
    }

    for node in &descriptor.nodes {
        let incoming_count = incoming_edges.get(node.id.as_str()).copied().unwrap_or(0);
        if node.id != descriptor.root_node && incoming_count != 1 {
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
    registered_tree_ids: &[&str],
) -> Result<(), AiManagerError> {
    validate_builtin_behavior_node_parameter_owners(tree_id, node)?;
    validate_subtree_target_parameter(tree_id, node, registered_tree_ids)?;

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
    registered_tree_ids: &[&str],
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
    if !registered_tree_ids.contains(&target_tree) {
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

pub(super) fn validate_blackboard_schema_descriptor(
    descriptor: &AiBlackboardSchemaDescriptor,
) -> Result<(), AiManagerError> {
    ensure_non_empty(&descriptor.id, "blackboard_schema.id")?;

    let mut keys = HashSet::new();
    for key in &descriptor.keys {
        ensure_non_empty(&key.key, "blackboard_key.key")?;
        ensure_non_empty(&key.value_type, "blackboard_key.value_type")?;
        if key.expected_value_type().is_none() {
            return Err(AiManagerError::UnknownBlackboardValueType {
                schema_id: descriptor.id.clone(),
                key: key.key.clone(),
                value_type: key.value_type.clone(),
            });
        }
        if !keys.insert(key.key.as_str()) {
            return Err(AiManagerError::DuplicateBlackboardKey {
                schema_id: descriptor.id.clone(),
                key: key.key.clone(),
            });
        }
    }

    Ok(())
}

pub(super) fn validate_blackboard_entries(
    schema: Option<&AiBlackboardSchemaDescriptor>,
    entries: &[AiBlackboardEntry],
) -> Result<(), AiManagerError> {
    let mut seen_entries = HashSet::new();
    for entry in entries {
        ensure_non_empty(&entry.key, "blackboard_entry.key")?;
        if !entry.value.is_finite() {
            return Err(AiManagerError::NonFiniteBlackboardValue {
                key: entry.key.clone(),
            });
        }
        if !seen_entries.insert(entry.key.as_str()) {
            return Err(AiManagerError::DuplicateBlackboardEntry {
                key: entry.key.clone(),
            });
        }
    }

    let Some(schema) = schema else {
        return Ok(());
    };

    for descriptor in &schema.keys {
        let matching_entry = entries.iter().find(|entry| entry.key == descriptor.key);
        if descriptor.required && matching_entry.is_none() {
            return Err(AiManagerError::MissingBlackboardKey {
                schema_id: schema.id.clone(),
                key: descriptor.key.clone(),
            });
        }
        if let Some(entry) = matching_entry {
            let expected = descriptor
                .expected_value_type()
                .expect("registered blackboard schema has normalized value types");
            let actual = entry.value.value_type();
            if expected != actual {
                return Err(AiManagerError::BlackboardValueTypeMismatch {
                    schema_id: schema.id.clone(),
                    key: entry.key.clone(),
                    expected: expected.as_str().to_string(),
                    actual: actual.as_str().to_string(),
                });
            }
        }
    }

    for entry in entries {
        if !schema
            .keys
            .iter()
            .any(|descriptor| descriptor.key == entry.key)
        {
            return Err(AiManagerError::UnknownBlackboardKey {
                schema_id: schema.id.clone(),
                key: entry.key.clone(),
            });
        }
    }

    Ok(())
}

pub(super) fn validate_perception_snapshot(
    entity: EntityId,
    snapshot: &AiPerceptionSnapshot,
) -> Result<(), AiManagerError> {
    if snapshot.agent != entity {
        return Err(AiManagerError::PerceptionAgentMismatch {
            expected: entity,
            actual: snapshot.agent,
        });
    }

    for stimulus in &snapshot.stimuli {
        if !stimulus.position.is_finite()
            || !stimulus.strength.is_finite()
            || !stimulus.age_seconds.is_finite()
        {
            return Err(AiManagerError::NonFinitePerceptionStimulus {
                source: stimulus.source,
            });
        }
    }

    Ok(())
}

fn ensure_non_empty(value: &str, field: &'static str) -> Result<(), AiManagerError> {
    if value.trim().is_empty() {
        return Err(AiManagerError::EmptyId { field });
    }
    Ok(())
}
