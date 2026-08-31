use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameterValue, AiBlackboardEntry, AiDecisionStatus, AiManagerError,
    AiPerceptionSnapshot,
};

use crate::blackboard::{
    BlackboardLayout, BlackboardObserver, BlackboardObserverSet, BlackboardSlot, BlackboardStore,
};
use crate::manager::parameters::{
    parse_task_result, ParallelPolicy, PARALLEL_FAILURE_POLICY_PARAMETER_KEY,
    PARALLEL_SUCCESS_POLICY_PARAMETER_KEY, SUBTREE_TARGET_PARAMETER_KEY, TASK_RESULT_PARAMETER_KEY,
};

use self::abort::{abort_active_root, process_observer_aborts, AbortRequest};
use self::condition::decorator_condition_passes;
use self::integration::{evaluate_integration_task, evaluate_task};
use self::selector::evaluate_selector;
use self::support::*;
use super::{
    BehaviorIntegrationHost, BehaviorNodeRuntime, BehaviorNodeSemantics, BehaviorNodeTickContext,
    CompiledBehaviorNode, CompiledBehaviorTree,
};

mod abort;
mod condition;
mod integration;
mod selector;
mod support;

#[cfg(test)]
#[path = "executor/parallel_allocation_tests.rs"]
mod parallel_allocation_tests;

#[cfg(test)]
#[path = "executor/node_state_allocation_tests.rs"]
mod node_state_allocation_tests;

#[cfg(test)]
#[path = "executor/observer_pass_allocation_tests.rs"]
mod observer_pass_allocation_tests;

#[cfg(test)]
#[path = "executor/tree_stack_allocation_tests.rs"]
mod tree_stack_allocation_tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BehaviorTreeExecution {
    pub(crate) status: AiDecisionStatus,
    pub(crate) active_node: Option<String>,
    pub(crate) diagnostic: Option<String>,
}

#[derive(Debug, Default)]
struct BehaviorTreeStack {
    ids: Vec<String>,
    depth: usize,
}

impl BehaviorTreeStack {
    fn reset(&mut self) {
        self.depth = 0;
    }

    fn push(&mut self, tree_id: &str) {
        if self.depth == self.ids.len() {
            self.ids.push(tree_id.to_string());
        } else {
            let slot = &mut self.ids[self.depth];
            slot.clear();
            slot.push_str(tree_id);
        }
        self.depth += 1;
    }

    fn pop(&mut self) {
        debug_assert!(self.depth > 0);
        self.depth -= 1;
    }

    fn contains(&self, tree_id: &str) -> bool {
        self.ids[..self.depth]
            .iter()
            .any(|candidate| candidate == tree_id)
    }
}

#[derive(Debug, Default)]
pub(crate) struct BehaviorTreeInstanceState {
    trees: std::collections::BTreeMap<String, Vec<BehaviorNodeRuntimeState>>,
    observers: std::collections::BTreeMap<String, BlackboardObserverSet>,
    observer_binding_root: Option<String>,
    observer_binding_schema: Option<String>,
    processed_observer_passes: std::collections::HashMap<String, u64>,
    observer_pass_epoch: u64,
    observer_scratch: Vec<BlackboardObserver>,
    abort_request_scratch: Vec<AbortRequest>,
    tree_stack_scratch: BehaviorTreeStack,
    root_tree: Option<String>,
    tick: u64,
}

#[derive(Debug, Default)]
struct BehaviorNodeRuntimeState {
    is_active: bool,
    elapsed_seconds: f32,
    cooldown_remaining: f32,
    loop_count: u32,
    selected_child: Option<u32>,
    active_child: Option<u32>,
    terminal_children: std::collections::BTreeMap<u32, BehaviorTreeExecution>,
    external_runtime: Option<Box<dyn BehaviorNodeRuntime>>,
}

impl BehaviorTreeInstanceState {
    fn node_state(
        &self,
        tree: &CompiledBehaviorTree,
        node_index: u32,
    ) -> Option<&BehaviorNodeRuntimeState> {
        self.trees
            .get(tree.id())
            .and_then(|states| states.get(node_index as usize))
    }

    fn node_mut(
        &mut self,
        tree: &CompiledBehaviorTree,
        node_index: u32,
    ) -> &mut BehaviorNodeRuntimeState {
        if !self.trees.contains_key(tree.id()) {
            let states = std::iter::repeat_with(BehaviorNodeRuntimeState::default)
                .take(tree.nodes().len())
                .collect();
            self.trees.insert(tree.id().to_string(), states);
        }
        let states = self
            .trees
            .get_mut(tree.id())
            .expect("behavior tree state was initialized");
        &mut states[node_index as usize]
    }

    fn bind_observers(
        &mut self,
        tree: &CompiledBehaviorTree,
        layout: &BlackboardLayout,
    ) -> Result<(), AiManagerError> {
        let requires_rebind = self
            .observers
            .get(tree.id())
            .is_none_or(|observers| observers.schema_id() != layout.schema_id());
        if requires_rebind {
            self.observers.insert(
                tree.id().to_string(),
                BlackboardObserverSet::resolve(tree, layout)?,
            );
        }
        Ok(())
    }

    fn next_observer_pass(&mut self) -> u64 {
        let next = self.observer_pass_epoch.wrapping_add(1);
        if next == 0 {
            self.processed_observer_passes.clear();
            self.observer_pass_epoch = 1;
        } else {
            self.observer_pass_epoch = next;
        }
        self.observer_pass_epoch
    }

    fn mark_observers_processed(&mut self, tree_id: &str, observer_pass: u64) -> bool {
        if let Some(processed_pass) = self.processed_observer_passes.get_mut(tree_id) {
            if *processed_pass == observer_pass {
                return false;
            }
            *processed_pass = observer_pass;
        } else {
            self.processed_observer_passes
                .insert(tree_id.to_string(), observer_pass);
        }
        true
    }

    pub(crate) fn invalidate_observer_bindings(&mut self) {
        self.observers.clear();
        self.observer_binding_root = None;
        self.observer_binding_schema = None;
    }
}

struct BehaviorTreeExecutionContext<'data, 'host> {
    blackboard: &'data [AiBlackboardEntry],
    perception: Option<&'data AiPerceptionSnapshot>,
    delta_seconds: f32,
    instance: &'data mut BehaviorTreeInstanceState,
    changed_slots: &'data [BlackboardSlot],
    observer_pass: u64,
    tree_descriptors: &'data [CompiledBehaviorTree],
    blackboard_store: Option<&'data BlackboardStore>,
    entity: u64,
    integration_host: Option<&'host mut dyn BehaviorIntegrationHost>,
}

impl BehaviorTreeExecutionContext<'_, '_> {
    fn dense_blackboard_value(
        &self,
        tree_id: &str,
        node_index: u32,
    ) -> Option<Option<zircon_runtime::core::framework::ai::AiBlackboardValue>> {
        let slot = self
            .instance
            .observers
            .get(tree_id)?
            .slot_for_node(node_index)?;
        Some(self.blackboard_store?.read(slot))
    }
}

pub(crate) fn evaluate_behavior_tree(
    descriptor: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
    delta_seconds: f32,
    blackboard_layout: Option<&BlackboardLayout>,
    blackboard_store: Option<&BlackboardStore>,
    changed_slots: &[BlackboardSlot],
    instance: &mut BehaviorTreeInstanceState,
    entity: u64,
    integration_host: Option<&mut dyn BehaviorIntegrationHost>,
) -> Result<BehaviorTreeExecution, AiManagerError> {
    if let Some(layout) = blackboard_layout {
        bind_reachable_observers(instance, descriptor, registered_trees, layout)?;
    }
    let observer_pass = instance.next_observer_pass();
    let mut tree_stack = std::mem::take(&mut instance.tree_stack_scratch);
    tree_stack.reset();
    let mut context = BehaviorTreeExecutionContext {
        blackboard,
        perception,
        delta_seconds,
        instance,
        changed_slots,
        observer_pass,
        tree_descriptors: registered_trees,
        blackboard_store,
        entity,
        integration_host,
    };
    if context.instance.root_tree.as_deref() != Some(descriptor.id()) {
        abort_active_root(&mut context);
        context.instance.trees.clear();
        context.instance.tick = 0;
        context.instance.root_tree = Some(descriptor.id().to_string());
    }
    let result = evaluate_behavior_tree_with_stack(
        descriptor,
        registered_trees,
        &mut context,
        &mut tree_stack,
    );
    context.instance.tree_stack_scratch = tree_stack;
    context.instance.tick = context.instance.tick.wrapping_add(1);
    Ok(result)
}

pub(crate) fn abort_behavior_tree_instance(
    registered_trees: &[CompiledBehaviorTree],
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
    delta_seconds: f32,
    instance: &mut BehaviorTreeInstanceState,
    entity: u64,
    integration_host: Option<&mut dyn BehaviorIntegrationHost>,
) {
    let mut context = BehaviorTreeExecutionContext {
        blackboard,
        perception,
        delta_seconds,
        instance,
        changed_slots: &[],
        observer_pass: 0,
        tree_descriptors: registered_trees,
        blackboard_store: None,
        entity,
        integration_host,
    };
    abort_active_root(&mut context);
    context.instance.trees.clear();
    context.instance.root_tree = None;
}

fn evaluate_behavior_tree_with_stack(
    descriptor: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    process_observer_aborts(descriptor, context);
    tree_stack.push(descriptor.id());
    let result = evaluate_node(0, descriptor, tree_descriptors, context, tree_stack);
    tree_stack.pop();
    result
}

fn evaluate_node(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let node = tree.node(node_index as usize);

    let result = match node.semantics() {
        BehaviorNodeSemantics::Selector => evaluate_selector(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::Sequence => evaluate_sequence(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::Parallel => evaluate_parallel(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::RandomSelector => evaluate_random_selector(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::BlackboardCondition => evaluate_decorator(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::Cooldown => evaluate_cooldown(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::TimeLimit => evaluate_time_limit(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::Loop => evaluate_loop(
            node_index,
            node,
            tree,
            tree_descriptors,
            context,
            tree_stack,
        ),
        BehaviorNodeSemantics::Inverter => {
            evaluate_inverter(node, tree, tree_descriptors, context, tree_stack)
        }
        BehaviorNodeSemantics::ForceResult => {
            evaluate_force_result(node, tree, tree_descriptors, context, tree_stack)
        }
        BehaviorNodeSemantics::UpdateBlackboardDistance => {
            evaluate_service(node, tree, tree_descriptors, context, tree_stack)
        }
        BehaviorNodeSemantics::Wait => evaluate_wait(node_index, node, tree, context),
        BehaviorNodeSemantics::RunSubtree => {
            evaluate_subtree(node, tree_descriptors, context, tree_stack)
        }
        BehaviorNodeSemantics::MoveTo
        | BehaviorNodeSemantics::PlayAnimation
        | BehaviorNodeSemantics::ScriptTask => {
            evaluate_integration_task(node_index, node, tree, context)
        }
        BehaviorNodeSemantics::SetBlackboard | BehaviorNodeSemantics::EmitEvent => {
            evaluate_task(node)
        }
        BehaviorNodeSemantics::External => evaluate_external(node_index, node, tree, context),
    };
    context.instance.node_mut(tree, node_index).is_active = matches!(
        &result.status,
        AiDecisionStatus::Running | AiDecisionStatus::Idle
    );
    result
}

fn evaluate_sequence(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let resume_child = context.instance.node_mut(tree, node_index).active_child;
    let mut last_succeeded = None;
    for child in tree
        .child_indices(node)
        .iter()
        .skip_while(|child| resume_child.is_some_and(|resume| **child != resume))
    {
        let result = evaluate_node(*child, tree, tree_descriptors, context, tree_stack);
        match &result.status {
            AiDecisionStatus::Succeeded => last_succeeded = Some(result),
            AiDecisionStatus::Running | AiDecisionStatus::Idle => {
                context.instance.node_mut(tree, node_index).active_child = Some(*child);
                return result;
            }
            _ => {
                context.instance.node_mut(tree, node_index).active_child = None;
                return result;
            }
        }
    }

    context.instance.node_mut(tree, node_index).active_child = None;

    last_succeeded.unwrap_or_else(|| BehaviorTreeExecution {
        status: AiDecisionStatus::Succeeded,
        active_node: Some(node.id().to_string()),
        diagnostic: None,
    })
}

fn evaluate_parallel(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    // This is a deterministic descriptor fold, not a thread/task scheduler. A later executor can
    // reuse the same policies when latent tasks and services become runtime state.
    let success_policy =
        parallel_policy(node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::All);
    let failure_policy =
        parallel_policy(node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::Any);
    let children = tree.child_indices(node);
    let child_count = children.len();
    let mut cached = {
        let state = context.instance.node_mut(tree, node_index);
        std::mem::take(&mut state.terminal_children)
    };
    let mut succeeded_child_count = 0_usize;
    let mut failed_child_count = 0_usize;
    let mut first_blocked_child = None;
    let mut last_succeeded_child = None;
    let mut last_failed_child = None;
    let mut first_running = None;
    let mut first_idle = None;
    for child in children {
        if let Some(result) = cached.get(child) {
            debug_assert!(is_terminal(&result.status));
            match &result.status {
                AiDecisionStatus::Succeeded => {
                    succeeded_child_count += 1;
                    last_succeeded_child = Some(*child);
                }
                AiDecisionStatus::Failed => {
                    failed_child_count += 1;
                    last_failed_child = Some(*child);
                }
                AiDecisionStatus::Blocked => {
                    first_blocked_child.get_or_insert(*child);
                }
                AiDecisionStatus::Running | AiDecisionStatus::Idle => {}
            }
            continue;
        }

        let result = evaluate_node(*child, tree, tree_descriptors, context, tree_stack);
        match &result.status {
            AiDecisionStatus::Succeeded => {
                succeeded_child_count += 1;
                last_succeeded_child = Some(*child);
                cached.insert(*child, result);
            }
            AiDecisionStatus::Failed => {
                failed_child_count += 1;
                last_failed_child = Some(*child);
                cached.insert(*child, result);
            }
            AiDecisionStatus::Blocked => {
                first_blocked_child.get_or_insert(*child);
                cached.insert(*child, result);
            }
            AiDecisionStatus::Running => {
                if first_running.is_none() {
                    first_running = Some(result);
                }
            }
            AiDecisionStatus::Idle => {
                if first_idle.is_none() {
                    first_idle = Some(result);
                }
            }
        }
    }

    if let Some(child) = first_blocked_child {
        let result = cached.remove(&child).expect("blocked child is cached");
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return result;
    }
    let success_policy_matches = match success_policy {
        ParallelPolicy::All => child_count > 0 && succeeded_child_count == child_count,
        ParallelPolicy::Any => succeeded_child_count > 0,
    };
    if success_policy_matches {
        let result = last_succeeded_child
            .and_then(|child| cached.remove(&child))
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Succeeded));
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return result;
    }
    let failure_policy_matches = match failure_policy {
        ParallelPolicy::All => child_count > 0 && failed_child_count == child_count,
        ParallelPolicy::Any => failed_child_count > 0,
    };
    if failure_policy_matches {
        let result = last_failed_child
            .and_then(|child| cached.remove(&child))
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Failed));
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return result;
    }
    if let Some(running) = first_running {
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children = cached;
        return running;
    }
    if let Some(idle) = first_idle {
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children = cached;
        return idle;
    }

    context
        .instance
        .node_mut(tree, node_index)
        .terminal_children = cached;
    node_result(node, AiDecisionStatus::Running)
}

fn evaluate_decorator(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let dense_value = context.dense_blackboard_value(tree.id(), node_index);
    if !decorator_condition_passes(
        node,
        context.blackboard,
        context.perception,
        dense_value.as_ref().map(Option::as_ref),
    ) {
        return BehaviorTreeExecution {
            status: AiDecisionStatus::Failed,
            active_node: Some(node.id().to_string()),
            diagnostic: None,
        };
    }

    let Some(child) = tree.child_indices(node).first() else {
        return BehaviorTreeExecution {
            status: AiDecisionStatus::Succeeded,
            active_node: Some(node.id().to_string()),
            diagnostic: None,
        };
    };
    evaluate_node(*child, tree, tree_descriptors, context, tree_stack)
}

fn evaluate_random_selector(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let children = tree.child_indices(node);
    if children.is_empty() {
        return node_result(node, AiDecisionStatus::Failed);
    }
    let selected = context
        .instance
        .node_mut(tree, node_index)
        .selected_child
        .unwrap_or_else(|| weighted_random_child(node, tree, children, context.instance.tick));
    context.instance.node_mut(tree, node_index).selected_child = Some(selected);
    let result = evaluate_node(selected, tree, tree_descriptors, context, tree_stack);
    if is_terminal(&result.status) {
        context.instance.node_mut(tree, node_index).selected_child = None;
    }
    result
}

fn evaluate_cooldown(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let delta_seconds = context.delta_seconds.max(0.0);
    let state = context.instance.node_mut(tree, node_index);
    if state.cooldown_remaining > 0.0 {
        state.cooldown_remaining = (state.cooldown_remaining - delta_seconds).max(0.0);
        return node_result(node, AiDecisionStatus::Failed);
    }
    let Some(child) = tree.child_indices(node).first().copied() else {
        return node_result(node, AiDecisionStatus::Succeeded);
    };
    let result = evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    if is_terminal(&result.status) {
        context
            .instance
            .node_mut(tree, node_index)
            .cooldown_remaining =
            scalar_parameter(node, &["cooldown_seconds", "duration_seconds"]).unwrap_or(0.0);
    }
    result
}

fn evaluate_time_limit(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let Some(child) = tree.child_indices(node).first().copied() else {
        return node_result(node, AiDecisionStatus::Succeeded);
    };
    let result = evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    if result.status == AiDecisionStatus::Running {
        let delta_seconds = context.delta_seconds.max(0.0);
        let state = context.instance.node_mut(tree, node_index);
        state.elapsed_seconds += delta_seconds;
        let limit = scalar_parameter(node, &["time_limit_seconds", "duration_seconds"])
            .unwrap_or(f32::INFINITY);
        if state.elapsed_seconds >= limit {
            state.elapsed_seconds = 0.0;
            return node_result(node, AiDecisionStatus::Failed);
        }
    } else if is_terminal(&result.status) {
        context.instance.node_mut(tree, node_index).elapsed_seconds = 0.0;
    }
    result
}

fn evaluate_loop(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let Some(child) = tree.child_indices(node).first().copied() else {
        return node_result(node, AiDecisionStatus::Succeeded);
    };
    let result = evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    match result.status {
        AiDecisionStatus::Succeeded => {
            let infinite = bool_parameter(node, "infinite").unwrap_or(false);
            let target = integer_parameter(node, "count").unwrap_or(1).max(1) as u32;
            let state = context.instance.node_mut(tree, node_index);
            state.loop_count = state.loop_count.saturating_add(1);
            if infinite || state.loop_count < target {
                node_result(node, AiDecisionStatus::Running)
            } else {
                state.loop_count = 0;
                result
            }
        }
        AiDecisionStatus::Failed | AiDecisionStatus::Blocked => {
            context.instance.node_mut(tree, node_index).loop_count = 0;
            result
        }
        _ => result,
    }
}

fn evaluate_inverter(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let Some(child) = tree.child_indices(node).first().copied() else {
        return node_result(node, AiDecisionStatus::Succeeded);
    };
    let mut result = evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    result.status = match result.status {
        AiDecisionStatus::Succeeded => AiDecisionStatus::Failed,
        AiDecisionStatus::Failed => AiDecisionStatus::Succeeded,
        status => status,
    };
    result
}

fn evaluate_force_result(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let Some(child) = tree.child_indices(node).first().copied() else {
        return node_result(node, AiDecisionStatus::Succeeded);
    };
    let mut result = evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    if is_terminal(&result.status) {
        result.status = parameter(node, "forced_result")
            .and_then(AiBehaviorNodeParameterValue::as_string)
            .and_then(parse_task_result)
            .unwrap_or(AiDecisionStatus::Succeeded);
    }
    result
}

fn evaluate_service(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    if let Some(child) = tree.child_indices(node).first().copied() {
        return evaluate_node(child, tree, tree_descriptors, context, tree_stack);
    }
    let status = parameter(node, "service_result")
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .and_then(parse_task_result)
        .unwrap_or(AiDecisionStatus::Succeeded);
    node_result(node, status)
}

fn evaluate_wait(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) -> BehaviorTreeExecution {
    if parameter(node, TASK_RESULT_PARAMETER_KEY).is_some() {
        return evaluate_task(node);
    }
    let Some(duration) = scalar_parameter(node, &["duration_seconds"]) else {
        return node_result(node, AiDecisionStatus::Running);
    };
    let delta_seconds = context.delta_seconds.max(0.0);
    let state = context.instance.node_mut(tree, node_index);
    state.elapsed_seconds += delta_seconds;
    if state.elapsed_seconds >= duration {
        state.elapsed_seconds = 0.0;
        node_result(node, AiDecisionStatus::Succeeded)
    } else {
        node_result(node, AiDecisionStatus::Running)
    }
}

fn evaluate_external(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) -> BehaviorTreeExecution {
    let Some(factory) = node.factory() else {
        return blocked(node.id(), "does not provide an external runtime factory");
    };
    let tick_context = BehaviorNodeTickContext::new(
        node.parameters(),
        context.blackboard,
        context.perception,
        context.delta_seconds,
    );
    let state = context.instance.node_mut(tree, node_index);
    let runtime = state
        .external_runtime
        .get_or_insert_with(|| factory.create());
    node_result(node, runtime.tick(&tick_context))
}

fn evaluate_subtree(
    node: &CompiledBehaviorNode,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let Some(target_tree_id) = parameter(node, SUBTREE_TARGET_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
    else {
        return blocked(
            node.id(),
            "subtree node does not declare a target behavior tree",
        );
    };
    let Some(target_tree) = tree_descriptors
        .iter()
        .find(|tree| tree.id() == target_tree_id)
    else {
        return blocked(
            node.id(),
            "subtree node references an unregistered behavior tree",
        );
    };
    if tree_stack.contains(target_tree_id) {
        return blocked(
            node.id(),
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
