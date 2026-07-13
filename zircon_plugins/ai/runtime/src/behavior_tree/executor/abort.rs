use zircon_runtime::core::framework::ai::{AiBehaviorAbortPolicy, AiBehaviorNodeParameterValue};

use super::{
    decorator_condition_passes, BehaviorNodeRuntimeState, BehaviorNodeSemantics,
    BehaviorNodeTickContext, BehaviorTreeExecutionContext, BehaviorTreeInstanceState,
    CompiledBehaviorTree, SUBTREE_TARGET_PARAMETER_KEY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AbortRequest {
    SelfSubtree { node_index: u32 },
    LowerPriority { observer_index: u32 },
}

impl AbortRequest {
    const fn priority(self) -> u32 {
        match self {
            Self::SelfSubtree { node_index } => node_index,
            Self::LowerPriority { observer_index } => observer_index,
        }
    }
}

pub(super) fn process_observer_aborts(
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_>,
) {
    if context.changed_slots.is_empty()
        || !context.processed_observers.insert(tree.id().to_string())
    {
        return;
    }
    let observers = context
        .instance
        .observers
        .get(tree.id())
        .map(|set| set.matching(context.changed_slots))
        .unwrap_or_default();
    let mut requests = Vec::new();
    for observer in observers {
        let node = tree.node(observer.node_index as usize);
        let dense_value = context.dense_blackboard_value(tree.id(), observer.node_index);
        let condition_passes = decorator_condition_passes(
            node,
            context.blackboard,
            context.perception,
            dense_value.as_ref().map(Option::as_ref),
        );
        if matches!(
            observer.policy,
            AiBehaviorAbortPolicy::Self_ | AiBehaviorAbortPolicy::Both
        ) && !condition_passes
            && context
                .instance
                .node_mut(tree, observer.node_index)
                .is_active
        {
            requests.push(AbortRequest::SelfSubtree {
                node_index: observer.node_index,
            });
        }
        if matches!(
            observer.policy,
            AiBehaviorAbortPolicy::LowerPriority | AiBehaviorAbortPolicy::Both
        ) && condition_passes
        {
            requests.push(AbortRequest::LowerPriority {
                observer_index: observer.node_index,
            });
        }
    }
    requests.sort_by_key(|request| request.priority());
    for request in requests {
        match request {
            AbortRequest::SelfSubtree { node_index } => {
                abort_subtree(tree, node_index, context);
            }
            AbortRequest::LowerPriority { observer_index } => {
                abort_lower_priority_branch(tree, observer_index, context);
            }
        }
    }
}

pub(super) fn abort_active_root(context: &mut BehaviorTreeExecutionContext<'_>) {
    let Some(root_tree_id) = context.instance.root_tree.clone() else {
        return;
    };
    let Some(root_tree) = context
        .tree_descriptors
        .iter()
        .find(|tree| tree.id() == root_tree_id)
        .cloned()
    else {
        return;
    };
    abort_subtree(&root_tree, 0, context);
}

fn abort_lower_priority_branch(
    tree: &CompiledBehaviorTree,
    observer_index: u32,
    context: &mut BehaviorTreeExecutionContext<'_>,
) {
    let Some((selector_index, observer_branch)) = selector_ancestor(tree, observer_index) else {
        return;
    };
    let active_branch = context.instance.node_mut(tree, selector_index).active_child;
    let Some(active_branch) = active_branch else {
        return;
    };
    let selector = tree.node(selector_index as usize);
    let children = tree.child_indices(selector);
    let observer_priority = children.iter().position(|child| *child == observer_branch);
    let active_priority = children.iter().position(|child| *child == active_branch);
    if !matches!((observer_priority, active_priority), (Some(observer), Some(active)) if active > observer)
    {
        return;
    }
    abort_subtree(tree, active_branch, context);
    clear_node_control_state(context.instance.node_mut(tree, selector_index));
    clear_ancestor_control_state(tree, selector_index, context.instance);
}

fn abort_subtree(
    tree: &CompiledBehaviorTree,
    root_index: u32,
    context: &mut BehaviorTreeExecutionContext<'_>,
) {
    let range = tree.node(root_index as usize).subtree_range(root_index);
    for node_index in range {
        let node = tree.node(node_index as usize);
        let subtree_target = (node.semantics() == BehaviorNodeSemantics::RunSubtree)
            .then(|| subtree_target(node))
            .flatten();
        let (was_active, runtime) = {
            let state = context.instance.node_mut(tree, node_index);
            let was_active = state.is_active;
            state.is_active = false;
            state.elapsed_seconds = 0.0;
            state.loop_count = 0;
            state.selected_child = None;
            state.active_child = None;
            state.terminal_children.clear();
            (was_active, state.external_runtime.take())
        };
        if was_active {
            if let Some(target_tree) = subtree_target.and_then(|target| {
                context
                    .tree_descriptors
                    .iter()
                    .find(|candidate| candidate.id() == target)
                    .cloned()
            }) {
                abort_subtree(&target_tree, 0, context);
            }
        }
        if was_active {
            let Some(mut runtime) = runtime else {
                continue;
            };
            let abort_context = BehaviorNodeTickContext::new(
                node.parameters(),
                context.blackboard,
                context.perception,
                context.delta_seconds,
            );
            runtime.on_abort(&abort_context);
        }
    }
}

fn clear_ancestor_control_state(
    tree: &CompiledBehaviorTree,
    node_index: u32,
    instance: &mut BehaviorTreeInstanceState,
) {
    let mut child = node_index;
    while let Some(parent) = parent_of(tree, child) {
        clear_node_control_state(instance.node_mut(tree, parent));
        child = parent;
    }
}

fn clear_node_control_state(state: &mut BehaviorNodeRuntimeState) {
    state.is_active = false;
    state.elapsed_seconds = 0.0;
    state.loop_count = 0;
    state.selected_child = None;
    state.active_child = None;
    state.terminal_children.clear();
}

fn subtree_target(node: &super::CompiledBehaviorNode) -> Option<&str> {
    node.parameters()
        .iter()
        .find(|parameter| parameter.key == SUBTREE_TARGET_PARAMETER_KEY)
        .and_then(|parameter| match &parameter.value {
            AiBehaviorNodeParameterValue::String(target) => Some(target.as_str()),
            _ => None,
        })
}

fn selector_ancestor(tree: &CompiledBehaviorTree, node_index: u32) -> Option<(u32, u32)> {
    let mut branch = node_index;
    while let Some(parent) = parent_of(tree, branch) {
        if tree.node(parent as usize).semantics() == BehaviorNodeSemantics::Selector {
            return Some((parent, branch));
        }
        branch = parent;
    }
    None
}

fn parent_of(tree: &CompiledBehaviorTree, node_index: u32) -> Option<u32> {
    tree.nodes().iter().enumerate().find_map(|(parent, node)| {
        tree.child_indices(node)
            .contains(&node_index)
            .then_some(parent as u32)
    })
}
