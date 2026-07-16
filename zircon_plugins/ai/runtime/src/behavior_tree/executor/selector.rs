use zircon_runtime::core::framework::ai::{AiBehaviorAbortPolicy, AiDecisionStatus};

use crate::behavior_tree::SelectorRecheckPolicy;
use crate::manager::parameters::{
    parse_task_result, ParallelPolicy, PARALLEL_FAILURE_POLICY_PARAMETER_KEY,
    PARALLEL_SUCCESS_POLICY_PARAMETER_KEY, TASK_RESULT_PARAMETER_KEY,
};

use super::abort::abort_subtree;
use super::condition::decorator_condition_passes;
use super::support::{parallel_policy, weighted_random_child};
use super::{
    evaluate_node, parameter, BehaviorNodeSemantics, BehaviorTreeExecution,
    BehaviorTreeExecutionContext, CompiledBehaviorNode, CompiledBehaviorTree,
};

pub(super) fn evaluate_selector(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
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
        let precedes_running_branch = !reached_resume;
        let requires_recheck = selector_branch_requires_recheck(*child, tree);
        let eligibility = (precedes_running_branch && requires_recheck)
            .then(|| selector_branch_eligibility(*child, tree, context));
        let mut preempted_before_evaluation = false;
        if eligibility == Some(SelectorBranchEligibility::Eligible) {
            if let Some(active_child) = resume_child {
                // A side-effect-free reactive guard selected this branch, so cleanup wins the race
                // with any integration task mutation below it.
                abort_subtree(tree, active_child, context);
                preempted_before_evaluation = true;
            }
        }
        let result = if precedes_running_branch
            && (!requires_recheck || eligibility == Some(SelectorBranchEligibility::Ineligible))
        {
            cached.get(child).cloned().unwrap_or_else(|| {
                evaluate_node(*child, tree, tree_descriptors, context, tree_stack)
            })
        } else {
            evaluate_node(*child, tree, tree_descriptors, context, tree_stack)
        };
        if precedes_running_branch
            && !preempted_before_evaluation
            && eligibility == Some(SelectorBranchEligibility::Deferred)
            && result.status != AiDecisionStatus::Failed
        {
            if let Some(active_child) = resume_child {
                // External reactive branches reveal eligibility only after their tick.
                abort_subtree(tree, active_child, context);
            }
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectorBranchEligibility {
    Eligible,
    Ineligible,
    Deferred,
}

fn selector_branch_eligibility(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> SelectorBranchEligibility {
    let node = tree.node(node_index as usize);
    if reactive_condition_requires_recheck(node) {
        return if selector_condition_passes(node_index, tree, context) {
            SelectorBranchEligibility::Eligible
        } else {
            SelectorBranchEligibility::Ineligible
        };
    }
    if node.selector_recheck_policy() == SelectorRecheckPolicy::RecheckWhileLowerPriorityRuns {
        return SelectorBranchEligibility::Deferred;
    }

    match node.semantics() {
        BehaviorNodeSemantics::Sequence => probe_sequence_children(node, tree, context),
        BehaviorNodeSemantics::Selector => probe_selector_children(node, tree, context),
        BehaviorNodeSemantics::RandomSelector => {
            probe_random_selector_child(node_index, node, tree, context)
        }
        BehaviorNodeSemantics::Parallel => probe_parallel_children(node_index, node, tree, context),
        BehaviorNodeSemantics::BlackboardCondition => {
            if !selector_condition_passes(node_index, tree, context) {
                return SelectorBranchEligibility::Ineligible;
            }
            tree.child_indices(node)
                .first()
                .filter(|child| selector_branch_requires_recheck(**child, tree))
                .map(|child| selector_branch_eligibility(*child, tree, context))
                .unwrap_or(SelectorBranchEligibility::Deferred)
        }
        _ => SelectorBranchEligibility::Deferred,
    }
}

fn probe_sequence_children(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> SelectorBranchEligibility {
    for child in tree.child_indices(node) {
        if selector_branch_requires_recheck(*child, tree) {
            return selector_branch_eligibility(*child, tree, context);
        }

        let child_node = tree.node(*child as usize);
        if child_node.semantics() != BehaviorNodeSemantics::BlackboardCondition {
            return SelectorBranchEligibility::Deferred;
        }
        if !selector_condition_passes(*child, tree, context) {
            return SelectorBranchEligibility::Ineligible;
        }
        if !tree.child_indices(child_node).is_empty() {
            return SelectorBranchEligibility::Deferred;
        }
    }
    SelectorBranchEligibility::Deferred
}

fn probe_selector_children(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> SelectorBranchEligibility {
    let mut saw_reactive_child = false;
    for child in tree.child_indices(node) {
        if !selector_branch_requires_recheck(*child, tree) {
            return if saw_reactive_child {
                SelectorBranchEligibility::Ineligible
            } else {
                SelectorBranchEligibility::Deferred
            };
        }
        saw_reactive_child = true;
        match selector_branch_eligibility(*child, tree, context) {
            SelectorBranchEligibility::Eligible => return SelectorBranchEligibility::Eligible,
            SelectorBranchEligibility::Deferred => {
                return SelectorBranchEligibility::Deferred;
            }
            SelectorBranchEligibility::Ineligible => {}
        }
    }
    SelectorBranchEligibility::Ineligible
}

fn probe_random_selector_child(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> SelectorBranchEligibility {
    let children = tree.child_indices(node);
    let Some(selected) = context
        .instance
        .node_state(tree, node_index)
        .and_then(|state| state.selected_child)
        .or_else(|| {
            (!children.is_empty())
                .then(|| weighted_random_child(node, tree, children, context.instance.tick))
        })
    else {
        return SelectorBranchEligibility::Ineligible;
    };
    if !selector_branch_requires_recheck(selected, tree) {
        return SelectorBranchEligibility::Ineligible;
    }
    selector_branch_eligibility(selected, tree, context)
}

fn probe_parallel_children(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> SelectorBranchEligibility {
    let mut eligible_children = Vec::new();
    let mut saw_deferred_child = false;
    let mut known_failed_reactive_children = 0;
    for child in tree.child_indices(node) {
        if !selector_branch_requires_recheck(*child, tree) {
            continue;
        }
        match selector_branch_eligibility(*child, tree, context) {
            SelectorBranchEligibility::Eligible => eligible_children.push(*child),
            SelectorBranchEligibility::Deferred => {
                saw_deferred_child = true;
            }
            SelectorBranchEligibility::Ineligible => {
                known_failed_reactive_children += 1;
            }
        }
    }
    if eligible_children.is_empty() {
        return if saw_deferred_child {
            SelectorBranchEligibility::Deferred
        } else {
            SelectorBranchEligibility::Ineligible
        };
    }

    let fixed_nonreactive = tree
        .child_indices(node)
        .iter()
        .filter(|child| !selector_branch_requires_recheck(**child, tree))
        .map(|child| fixed_parallel_child_status(node_index, *child, tree, context))
        .collect::<Option<Vec<_>>>();
    let Some(fixed_nonreactive) = fixed_nonreactive else {
        // A reactive guard is already true. Preempt before evaluating unknown siblings so an
        // integration task cannot mutate the host ahead of lower-branch cleanup.
        return SelectorBranchEligibility::Eligible;
    };
    if fixed_nonreactive
        .iter()
        .any(|status| *status == AiDecisionStatus::Blocked)
    {
        return SelectorBranchEligibility::Ineligible;
    }

    let success_policy =
        parallel_policy(node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::All);
    if success_policy == ParallelPolicy::Any
        && (fixed_nonreactive
            .iter()
            .any(|status| *status == AiDecisionStatus::Succeeded)
            || eligible_children
                .iter()
                .any(|child| reactive_branch_guarantees_success(*child, tree, context)))
    {
        return SelectorBranchEligibility::Eligible;
    }

    let failure_policy =
        parallel_policy(node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::Any);
    if failure_policy == ParallelPolicy::Any
        && (known_failed_reactive_children > 0
            || fixed_nonreactive
                .iter()
                .any(|status| *status == AiDecisionStatus::Failed))
    {
        return SelectorBranchEligibility::Ineligible;
    }
    if failure_policy == ParallelPolicy::All
        && eligible_children
            .iter()
            .all(|child| !reactive_branch_can_avoid_failure(*child, tree, context))
        && fixed_nonreactive
            .iter()
            .all(|status| *status == AiDecisionStatus::Failed)
    {
        return SelectorBranchEligibility::Ineligible;
    }

    SelectorBranchEligibility::Eligible
}

fn fixed_parallel_child_status(
    parallel_index: u32,
    child_index: u32,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> Option<AiDecisionStatus> {
    context
        .instance
        .node_state(tree, parallel_index)
        .and_then(|state| state.terminal_children.get(&child_index))
        .map(|result| result.status.clone())
        .or_else(|| explicit_task_result(child_index, tree))
}

fn explicit_task_result(node_index: u32, tree: &CompiledBehaviorTree) -> Option<AiDecisionStatus> {
    parameter(tree.node(node_index as usize), TASK_RESULT_PARAMETER_KEY)
        .and_then(|value| value.as_string())
        .and_then(parse_task_result)
}

fn reactive_branch_guarantees_success(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> bool {
    let node = tree.node(node_index as usize);
    if node.semantics() != BehaviorNodeSemantics::BlackboardCondition
        || !selector_condition_passes(node_index, tree, context)
    {
        return false;
    }
    tree.child_indices(node).first().map_or(true, |child| {
        explicit_task_result(*child, tree) == Some(AiDecisionStatus::Succeeded)
    })
}

fn reactive_branch_can_avoid_failure(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> bool {
    let node = tree.node(node_index as usize);
    if node.semantics() != BehaviorNodeSemantics::BlackboardCondition
        || !selector_condition_passes(node_index, tree, context)
    {
        return false;
    }
    tree.child_indices(node).first().is_none_or(|child| {
        !matches!(
            explicit_task_result(*child, tree),
            Some(AiDecisionStatus::Failed | AiDecisionStatus::Blocked)
        )
    })
}

fn selector_condition_passes(
    node_index: u32,
    tree: &CompiledBehaviorTree,
    context: &BehaviorTreeExecutionContext<'_, '_>,
) -> bool {
    let node = tree.node(node_index as usize);
    if node.semantics() != BehaviorNodeSemantics::BlackboardCondition {
        return false;
    }
    let dense_value = context.dense_blackboard_value(tree.id(), node_index);
    decorator_condition_passes(
        node,
        context.blackboard,
        context.perception,
        dense_value.as_ref().map(Option::as_ref),
    )
}

fn selector_branch_requires_recheck(node_index: u32, tree: &CompiledBehaviorTree) -> bool {
    let node = tree.node(node_index as usize);
    if node.semantics() == BehaviorNodeSemantics::BlackboardCondition {
        return reactive_condition_requires_recheck(node);
    }
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

fn reactive_condition_requires_recheck(node: &CompiledBehaviorNode) -> bool {
    if node.semantics() != BehaviorNodeSemantics::BlackboardCondition {
        return false;
    }
    if parameter(node, "blackboard_key").is_some() {
        return matches!(
            node.abort_policy(),
            AiBehaviorAbortPolicy::LowerPriority | AiBehaviorAbortPolicy::Both
        );
    }
    node.selector_recheck_policy() == SelectorRecheckPolicy::RecheckWhileLowerPriorityRuns
}
