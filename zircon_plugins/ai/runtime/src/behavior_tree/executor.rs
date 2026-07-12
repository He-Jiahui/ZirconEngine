use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameterValue, AiBlackboardEntry, AiDecisionStatus, AiPerceptionSnapshot,
};

use crate::manager::parameters::{
    parse_parallel_policy, parse_task_result, ParallelPolicy,
    PARALLEL_FAILURE_POLICY_PARAMETER_KEY, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY,
    SUBTREE_TARGET_PARAMETER_KEY, TASK_RESULT_PARAMETER_KEY,
};

use self::condition::decorator_condition_passes;
use super::{
    BehaviorNodeRuntime, BehaviorNodeSemantics, BehaviorNodeTickContext, CompiledBehaviorNode,
    CompiledBehaviorTree, SelectorRecheckPolicy,
};

mod condition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BehaviorTreeExecution {
    pub(crate) status: AiDecisionStatus,
    pub(crate) active_node: Option<String>,
    pub(crate) diagnostic: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct BehaviorTreeInstanceState {
    trees: std::collections::BTreeMap<String, Vec<BehaviorNodeRuntimeState>>,
    root_tree: Option<String>,
    tick: u64,
}

#[derive(Debug, Default)]
struct BehaviorNodeRuntimeState {
    elapsed_seconds: f32,
    cooldown_remaining: f32,
    loop_count: u32,
    selected_child: Option<u32>,
    active_child: Option<u32>,
    terminal_children: std::collections::BTreeMap<u32, BehaviorTreeExecution>,
    external_runtime: Option<Box<dyn BehaviorNodeRuntime>>,
}

impl BehaviorTreeInstanceState {
    fn begin_root(&mut self, tree: &CompiledBehaviorTree) {
        if self.root_tree.as_deref() != Some(tree.id()) {
            self.trees.clear();
            self.tick = 0;
            self.root_tree = Some(tree.id().to_string());
        }
    }

    fn node_mut(
        &mut self,
        tree: &CompiledBehaviorTree,
        node_index: u32,
    ) -> &mut BehaviorNodeRuntimeState {
        let states = self.trees.entry(tree.id().to_string()).or_insert_with(|| {
            std::iter::repeat_with(BehaviorNodeRuntimeState::default)
                .take(tree.nodes().len())
                .collect()
        });
        &mut states[node_index as usize]
    }
}

struct BehaviorTreeExecutionContext<'a> {
    blackboard: &'a [AiBlackboardEntry],
    perception: Option<&'a AiPerceptionSnapshot>,
    delta_seconds: f32,
    instance: &'a mut BehaviorTreeInstanceState,
}

pub(crate) fn evaluate_behavior_tree(
    descriptor: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
    blackboard: &[AiBlackboardEntry],
    perception: Option<&AiPerceptionSnapshot>,
    delta_seconds: f32,
    instance: &mut BehaviorTreeInstanceState,
) -> BehaviorTreeExecution {
    instance.begin_root(descriptor);
    let mut context = BehaviorTreeExecutionContext {
        blackboard,
        perception,
        delta_seconds,
        instance,
    };
    let result = evaluate_behavior_tree_with_stack(
        descriptor,
        registered_trees,
        &mut context,
        &mut Vec::new(),
    );
    context.instance.tick = context.instance.tick.wrapping_add(1);
    result
}

fn evaluate_behavior_tree_with_stack(
    descriptor: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    tree_stack.push(descriptor.id().to_string());
    let result = evaluate_node(0, descriptor, tree_descriptors, context, tree_stack);
    tree_stack.pop();
    result
}

fn evaluate_node(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let node = tree.node(node_index as usize);

    match node.semantics() {
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
        BehaviorNodeSemantics::BlackboardCondition => {
            evaluate_decorator(node, tree, tree_descriptors, context, tree_stack)
        }
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
        | BehaviorNodeSemantics::SetBlackboard
        | BehaviorNodeSemantics::EmitEvent
        | BehaviorNodeSemantics::ScriptTask => evaluate_task(node),
        BehaviorNodeSemantics::External => evaluate_external(node_index, node, tree, context),
    }
}

fn evaluate_selector(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    let cached = context
        .instance
        .node_mut(tree, node_index)
        .terminal_children
        .clone();
    let resume_child = context.instance.node_mut(tree, node_index).active_child;
    let mut reached_resume = resume_child.is_none();
    let mut last_failed = None;
    for child in tree.child_indices(node) {
        if resume_child == Some(*child) {
            reached_resume = true;
        }
        let result = if !reached_resume && !selector_branch_requires_recheck(*child, tree) {
            cached.get(child).cloned().unwrap_or_else(|| {
                evaluate_node(*child, tree, tree_descriptors, context, tree_stack)
            })
        } else {
            evaluate_node(*child, tree, tree_descriptors, context, tree_stack)
        };
        match &result.status {
            AiDecisionStatus::Failed => {
                context
                    .instance
                    .node_mut(tree, node_index)
                    .terminal_children
                    .insert(*child, result.clone());
                last_failed = Some(result);
            }
            AiDecisionStatus::Running | AiDecisionStatus::Idle => {
                context.instance.node_mut(tree, node_index).active_child = Some(*child);
                return result;
            }
            _ => {
                context.instance.node_mut(tree, node_index).active_child = None;
                context
                    .instance
                    .node_mut(tree, node_index)
                    .terminal_children
                    .clear();
                return result;
            }
        }
    }

    context.instance.node_mut(tree, node_index).active_child = None;
    context
        .instance
        .node_mut(tree, node_index)
        .terminal_children
        .clear();

    last_failed.unwrap_or_else(|| BehaviorTreeExecution {
        status: AiDecisionStatus::Failed,
        active_node: Some(node.id().to_string()),
        diagnostic: None,
    })
}

fn selector_branch_requires_recheck(node_index: u32, tree: &CompiledBehaviorTree) -> bool {
    let node = tree.node(node_index as usize);
    if node.selector_recheck_policy() == SelectorRecheckPolicy::RecheckWhileLowerPriorityRuns {
        return true;
    }
    match node.semantics() {
        BehaviorNodeSemantics::Selector
        | BehaviorNodeSemantics::Sequence
        | BehaviorNodeSemantics::Parallel
        | BehaviorNodeSemantics::RandomSelector => tree
            .child_indices(node)
            .iter()
            .any(|child| selector_branch_requires_recheck(*child, tree)),
        _ => false,
    }
}

fn evaluate_sequence(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    // This is a deterministic descriptor fold, not a thread/task scheduler. A later executor can
    // reuse the same policies when latent tasks and services become runtime state.
    let success_policy =
        parallel_policy(node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::All);
    let failure_policy =
        parallel_policy(node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::Any);
    let cached = context
        .instance
        .node_mut(tree, node_index)
        .terminal_children
        .clone();
    let mut child_results = Vec::with_capacity(tree.child_indices(node).len());
    for child in tree.child_indices(node) {
        let result = cached
            .get(child)
            .cloned()
            .unwrap_or_else(|| evaluate_node(*child, tree, tree_descriptors, context, tree_stack));
        if is_terminal(&result.status) {
            context
                .instance
                .node_mut(tree, node_index)
                .terminal_children
                .insert(*child, result.clone());
        }
        child_results.push(result);
    }

    if let Some(blocked) = first_status(&child_results, AiDecisionStatus::Blocked) {
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return blocked.clone();
    }
    if parallel_policy_matches(&child_results, AiDecisionStatus::Succeeded, success_policy) {
        let result = selected_parallel_result(&child_results, AiDecisionStatus::Succeeded)
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Succeeded));
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return result;
    }
    if parallel_policy_matches(&child_results, AiDecisionStatus::Failed, failure_policy) {
        let result = selected_parallel_result(&child_results, AiDecisionStatus::Failed)
            .unwrap_or_else(|| node_result(node, AiDecisionStatus::Failed));
        context
            .instance
            .node_mut(tree, node_index)
            .terminal_children
            .clear();
        return result;
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
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
) -> BehaviorTreeExecution {
    if !decorator_condition_passes(node, context.blackboard, context.perception) {
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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

fn weighted_random_child(
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
            let id_key = format!("weight.{}", tree.node(*child as usize).id());
            let position_key = format!("weight_{position}");
            scalar_parameter(node, &[id_key.as_str(), position_key.as_str()])
                .unwrap_or(1.0)
                .max(0.0)
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

fn evaluate_cooldown(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
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

fn evaluate_task(node: &CompiledBehaviorNode) -> BehaviorTreeExecution {
    let status = parameter(node, TASK_RESULT_PARAMETER_KEY)
        .and_then(AiBehaviorNodeParameterValue::as_string)
        .and_then(parse_task_result)
        .unwrap_or(AiDecisionStatus::Running);

    BehaviorTreeExecution {
        status,
        active_node: Some(node.id().to_string()),
        diagnostic: None,
    }
}

fn evaluate_external(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_>,
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
    context: &mut BehaviorTreeExecutionContext<'_>,
    tree_stack: &mut Vec<String>,
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
    if tree_stack
        .iter()
        .any(|tree_id| tree_id.as_str() == target_tree_id)
    {
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

fn parameter<'a>(
    node: &'a CompiledBehaviorNode,
    key: &str,
) -> Option<&'a AiBehaviorNodeParameterValue> {
    node.parameters()
        .iter()
        .find(|parameter| parameter.key == key)
        .map(|parameter| &parameter.value)
}

fn scalar_parameter(node: &CompiledBehaviorNode, keys: &[&str]) -> Option<f32> {
    keys.iter().find_map(|key| match parameter(node, key) {
        Some(AiBehaviorNodeParameterValue::Scalar(value)) => Some(*value),
        _ => None,
    })
}

fn integer_parameter(node: &CompiledBehaviorNode, key: &str) -> Option<i64> {
    match parameter(node, key) {
        Some(AiBehaviorNodeParameterValue::Integer(value)) => Some(*value),
        _ => None,
    }
}

fn bool_parameter(node: &CompiledBehaviorNode, key: &str) -> Option<bool> {
    parameter(node, key).and_then(AiBehaviorNodeParameterValue::as_bool)
}

fn is_terminal(status: &AiDecisionStatus) -> bool {
    matches!(
        status,
        AiDecisionStatus::Succeeded | AiDecisionStatus::Failed | AiDecisionStatus::Blocked
    )
}

fn parallel_policy(node: &CompiledBehaviorNode, key: &str) -> Option<ParallelPolicy> {
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

fn node_result(node: &CompiledBehaviorNode, status: AiDecisionStatus) -> BehaviorTreeExecution {
    BehaviorTreeExecution {
        status,
        active_node: Some(node.id().to_string()),
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
