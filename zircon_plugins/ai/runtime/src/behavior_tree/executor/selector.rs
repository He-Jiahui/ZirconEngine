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

#[cfg(test)]
#[path = "selector/allocation_tests.rs"]
mod allocation_tests;

pub(super) fn evaluate_selector(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    tree_descriptors: &[CompiledBehaviorTree],
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
    tree_stack: &mut super::BehaviorTreeStack,
) -> BehaviorTreeExecution {
    let (mut cached, resume_child) = {
        let state = context.instance.node_mut(tree, node_index);
        (
            std::mem::take(&mut state.terminal_children),
            state.active_child,
        )
    };
    let mut reached_resume = resume_child.is_none();
    let mut last_failed_child = None;
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
            cached.remove(child).unwrap_or_else(|| {
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
        if result.status == AiDecisionStatus::Failed {
            cached.insert(*child, result);
            last_failed_child = Some(*child);
        } else if matches!(
            &result.status,
            AiDecisionStatus::Running | AiDecisionStatus::Idle
        ) {
            let state = context.instance.node_mut(tree, node_index);
            state.active_child = Some(*child);
            state.terminal_children = cached;
            return result;
        } else {
            let state = context.instance.node_mut(tree, node_index);
            state.active_child = None;
            state.terminal_children.clear();
            return result;
        }
    }

    let last_failed = last_failed_child.and_then(|child| cached.remove(&child));
    let state = context.instance.node_mut(tree, node_index);
    state.active_child = None;
    state.terminal_children.clear();

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
    let success_policy =
        parallel_policy(node, PARALLEL_SUCCESS_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::All);
    let failure_policy =
        parallel_policy(node, PARALLEL_FAILURE_POLICY_PARAMETER_KEY).unwrap_or(ParallelPolicy::Any);
    let mut eligible_child_count = 0_usize;
    let mut eligible_guarantees_success = false;
    let mut all_eligible_must_fail = true;
    let mut saw_deferred_child = false;
    let mut known_failed_reactive_children = 0;
    for child in tree.child_indices(node) {
        if !selector_branch_requires_recheck(*child, tree) {
            continue;
        }
        match selector_branch_eligibility(*child, tree, context) {
            SelectorBranchEligibility::Eligible => {
                eligible_child_count += 1;
                if success_policy == ParallelPolicy::Any
                    && reactive_branch_guarantees_success(*child, tree, context)
                {
                    eligible_guarantees_success = true;
                }
                if failure_policy == ParallelPolicy::All
                    && reactive_branch_can_avoid_failure(*child, tree, context)
                {
                    all_eligible_must_fail = false;
                }
            }
            SelectorBranchEligibility::Deferred => {
                saw_deferred_child = true;
            }
            SelectorBranchEligibility::Ineligible => {
                known_failed_reactive_children += 1;
            }
        }
    }
    if eligible_child_count == 0 {
        return if saw_deferred_child {
            SelectorBranchEligibility::Deferred
        } else {
            SelectorBranchEligibility::Ineligible
        };
    }

    let mut fixed_has_blocked = false;
    let mut fixed_has_succeeded = false;
    let mut fixed_has_failed = false;
    let mut fixed_all_failed = true;
    for child in tree
        .child_indices(node)
        .iter()
        .filter(|child| !selector_branch_requires_recheck(**child, tree))
    {
        let Some(status) = fixed_parallel_child_status(node_index, *child, tree, context) else {
            // A reactive guard is already true. Preempt before evaluating unknown siblings so an
            // integration task cannot mutate the host ahead of lower-branch cleanup.
            return SelectorBranchEligibility::Eligible;
        };
        fixed_has_blocked |= status == AiDecisionStatus::Blocked;
        fixed_has_succeeded |= status == AiDecisionStatus::Succeeded;
        fixed_has_failed |= status == AiDecisionStatus::Failed;
        fixed_all_failed &= status == AiDecisionStatus::Failed;
    }
    if fixed_has_blocked {
        return SelectorBranchEligibility::Ineligible;
    }

    if success_policy == ParallelPolicy::Any && (fixed_has_succeeded || eligible_guarantees_success)
    {
        return SelectorBranchEligibility::Eligible;
    }

    if failure_policy == ParallelPolicy::Any
        && (known_failed_reactive_children > 0 || fixed_has_failed)
    {
        return SelectorBranchEligibility::Ineligible;
    }
    if failure_policy == ParallelPolicy::All && all_eligible_must_fail && fixed_all_failed {
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
