use zircon_runtime::core::framework::ai::{
    AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiManagerError,
};

use crate::behavior_tree::{
    compile_behavior_tree_with_catalog, BehaviorNodeCategory, BehaviorTreeCompileError,
};

use super::state::RegisteredBehaviorTree;
use super::validation::validate_behavior_tree_descriptor;
use super::DefaultAiManager;

pub(super) fn register(
    manager: &DefaultAiManager,
    descriptor: AiBehaviorTreeDescriptor,
) -> Result<AiBehaviorTreeId, AiManagerError> {
    let mut state = manager.lock_state();
    if state
        .behavior_trees
        .iter()
        .any(|entry| entry.descriptor.id == descriptor.id)
    {
        return Err(AiManagerError::DuplicateId { id: descriptor.id });
    }
    {
        let registered_tree_ids = state
            .behavior_trees
            .iter()
            .map(|entry| entry.descriptor.id.as_str())
            .collect::<Vec<_>>();
        validate_behavior_tree_descriptor(&descriptor, &registered_tree_ids)?;
    }
    let catalog = manager
        .behavior_node_catalog
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .snapshot();
    let compiled = compile_behavior_tree_with_catalog(&descriptor, &catalog)
        .map_err(|error| compile_error(&descriptor.id, error))?;

    state.next_behavior_tree_id += 1;
    let id = AiBehaviorTreeId::new(state.next_behavior_tree_id);
    state.behavior_trees.push(RegisteredBehaviorTree {
        id,
        descriptor,
        compiled,
    });
    state.rebuild_compiled_behavior_tree_generation();
    Ok(id)
}

fn compile_error(tree_id: &str, error: BehaviorTreeCompileError) -> AiManagerError {
    match error {
        BehaviorTreeCompileError::EmptyTreeId => AiManagerError::EmptyId {
            field: "behavior_tree.id",
        },
        BehaviorTreeCompileError::MissingRoot { node_id } => AiManagerError::MissingRootNode {
            tree_id: tree_id.to_string(),
            root_node: node_id,
        },
        BehaviorTreeCompileError::DuplicateNodeId { node_id } => {
            AiManagerError::DuplicateId { id: node_id }
        }
        BehaviorTreeCompileError::MissingChild { node_id, child_id } => {
            AiManagerError::MissingChildNode {
                tree_id: tree_id.to_string(),
                node_id,
                child_id,
            }
        }
        BehaviorTreeCompileError::MultipleParents { node_id } => {
            invalid_topology(tree_id, node_id, "node has multiple parents")
        }
        BehaviorTreeCompileError::Cycle { node_id } => {
            invalid_topology(tree_id, node_id, "node participates in a cycle")
        }
        BehaviorTreeCompileError::UnreachableNode { node_id } => {
            invalid_topology(tree_id, node_id, "node is not reachable from root")
        }
        BehaviorTreeCompileError::UnknownImplementation {
            node_id,
            implementation,
        } => AiManagerError::UnknownBehaviorNodeImplementation {
            tree_id: tree_id.to_string(),
            node_id,
            implementation,
        },
        BehaviorTreeCompileError::MissingCatalogDescriptor {
            node_id,
            implementation,
        } => AiManagerError::BehaviorNodeCatalogDescriptorMissing {
            tree_id: tree_id.to_string(),
            node_id,
            implementation,
        },
        BehaviorTreeCompileError::ImplementationCategoryMismatch {
            node_id,
            implementation,
            expected,
            actual,
        } => AiManagerError::BehaviorNodeImplementationCategoryMismatch {
            tree_id: tree_id.to_string(),
            node_id,
            implementation,
            expected: category_name(expected),
            actual: category_name(actual),
        },
        BehaviorTreeCompileError::StandardCatalog(_) => {
            AiManagerError::StandardBehaviorNodeCatalogUnavailable {
                tree_id: tree_id.to_string(),
            }
        }
    }
}

fn invalid_topology(tree_id: &str, node_id: String, reason: &'static str) -> AiManagerError {
    AiManagerError::InvalidBehaviorTreeTopology {
        tree_id: tree_id.to_string(),
        node_id,
        reason,
    }
}

fn category_name(category: BehaviorNodeCategory) -> &'static str {
    match category {
        BehaviorNodeCategory::Composite => "composite",
        BehaviorNodeCategory::Decorator => "decorator",
        BehaviorNodeCategory::Service => "service",
        BehaviorNodeCategory::Task => "task",
    }
}

pub(super) fn descriptors(manager: &DefaultAiManager) -> Vec<AiBehaviorTreeDescriptor> {
    manager
        .lock_state()
        .behavior_trees
        .iter()
        .map(|entry| entry.descriptor.clone())
        .collect()
}

pub(super) fn revoke_node_owner(
    manager: &DefaultAiManager,
    owner: zircon_runtime::plugin::PluginModuleId,
) -> Vec<crate::behavior_tree::BehaviorNodeSlot> {
    let revocation_guard = manager.behavior_node_execution_gate.revoke_and_wait(owner);
    let mut state = manager.lock_state();
    let removed = manager
        .behavior_node_catalog
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove_owned_by(owner);
    if removed.is_empty() {
        drop(state);
        drop(revocation_guard);
        return removed;
    }

    let retired_tree_ids = state
        .behavior_trees
        .iter()
        .filter(|tree| tree.compiled.uses_any_implementation(&removed))
        .map(|tree| tree.id)
        .collect::<Vec<_>>();
    state
        .behavior_trees
        .retain(|tree| !retired_tree_ids.contains(&tree.id));
    state.rebuild_compiled_behavior_tree_generation();
    if !retired_tree_ids.is_empty() {
        for instance in state.behavior_tree_instances.values_mut() {
            instance.invalidate_observer_bindings();
        }
    }
    let retired_agents = state
        .active_behavior_trees
        .iter()
        .filter(|(_, active)| retired_tree_ids.contains(&active.behavior_tree))
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();
    for agent in retired_agents {
        state.active_behavior_trees.remove(&agent);
        state.behavior_tree_instances.remove(&agent);
        state.last_reports.remove(&agent);
    }
    drop(state);
    drop(revocation_guard);
    removed
}
