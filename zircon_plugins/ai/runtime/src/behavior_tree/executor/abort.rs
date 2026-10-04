//! 黑板观察者按节点优先级触发中止；清理只通知仍活跃的任务与外部运行时。

use zircon_runtime::core::framework::ai::{AiBehaviorAbortPolicy, AiBehaviorNodeParameterValue};

use crate::behavior_tree::BehaviorIntegrationTaskContext;

use super::{
    decorator_condition_passes, BehaviorNodeRuntimeState, BehaviorNodeSemantics,
    BehaviorNodeTickContext, BehaviorTreeExecutionContext, BehaviorTreeInstanceState,
    CompiledBehaviorTree, SUBTREE_TARGET_PARAMETER_KEY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AbortRequest {
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
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) {
    // 同一观察轮次内每棵树只处理一次变更槽位，避免子树重入时重复中止。
    let observer_pass = context.observer_pass;
    if context.changed_slots.is_empty()
        || !context
            .instance
            .mark_observers_processed(tree.id(), observer_pass)
    {
        return;
    }
    let mut observers = std::mem::take(&mut context.instance.observer_scratch);
    observers.clear();
    if let Some(set) = context.instance.observers.get(tree.id()) {
        set.append_matching(context.changed_slots, &mut observers);
    }
    let mut requests = std::mem::take(&mut context.instance.abort_request_scratch);
    requests.clear();
    requests.reserve(observers.len());
    for observer in observers.iter().copied() {
        let node = tree.node(observer.node_index as usize);
        let dense_value = context.dense_blackboard_value(tree.id(), observer.node_index);
        let condition_passes = decorator_condition_passes(
            node,
            context.blackboard,
            context.perception,
            dense_value.as_ref().map(Option::as_ref),
            context.effects.blackboard_overlay(),
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
    context.instance.observer_scratch = observers;
    // Each compiled node owns one observer, so emitted priorities are unique.
    requests.sort_unstable_by_key(|request| request.priority());
    for request in requests.iter().copied() {
        match request {
            AbortRequest::SelfSubtree { node_index } => {
                abort_subtree(tree, node_index, context);
            }
            AbortRequest::LowerPriority { observer_index } => {
                abort_lower_priority_branch(tree, observer_index, context);
            }
        }
    }
    context.instance.abort_request_scratch = requests;
}

pub(super) fn abort_active_root(context: &mut BehaviorTreeExecutionContext<'_, '_>) {
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
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
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

pub(super) fn abort_subtree(
    tree: &CompiledBehaviorTree,
    root_index: u32,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) {
    // 编译后的子树节点占据连续索引；重置全部状态，仅对先前活跃节点调用撤销钩子。
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
            if matches!(
                node.semantics(),
                BehaviorNodeSemantics::MoveTo
                    | BehaviorNodeSemantics::PlayAnimation
                    | BehaviorNodeSemantics::ScriptTask
            ) {
                let abort_context = BehaviorIntegrationTaskContext {
                    node_id: node.id(),
                    parameters: node.parameters(),
                    entity: context.entity,
                    delta_seconds: context.delta_seconds,
                    started: false,
                };
                if let Some(host) = context.integration_host.as_deref_mut() {
                    host.abort(&abort_context);
                }
            }
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
    tree.parent_index(node_index)
}

#[cfg(test)]
#[path = "tests/abort_parent_index_contract_tests.rs"]
mod parent_index_contract_tests;

#[cfg(test)]
#[path = "tests/abort_abort_request_performance_tests.rs"]
mod abort_request_performance_tests;
