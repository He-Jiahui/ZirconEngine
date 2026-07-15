use crate::behavior_tree::{
    BehaviorIntegrationTaskContext, BehaviorNodeSemantics, CompiledBehaviorNode,
    CompiledBehaviorTree,
};
use crate::manager::parameters::{parse_task_result, TASK_RESULT_PARAMETER_KEY};
use zircon_runtime::core::framework::ai::{AiBehaviorNodeParameterValue, AiDecisionStatus};

use super::{blocked, parameter, BehaviorTreeExecution, BehaviorTreeExecutionContext};

pub(super) fn evaluate_task(node: &CompiledBehaviorNode) -> BehaviorTreeExecution {
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

pub(super) fn evaluate_integration_task(
    node_index: u32,
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) -> BehaviorTreeExecution {
    if parameter(node, TASK_RESULT_PARAMETER_KEY).is_some() {
        return evaluate_task(node);
    }
    let started = !context.instance.node_mut(tree, node_index).is_active;
    let request = BehaviorIntegrationTaskContext {
        node_id: node.id(),
        parameters: node.parameters(),
        entity: context.entity,
        delta_seconds: context.delta_seconds,
        started,
    };
    let Some(host) = context.integration_host.as_deref_mut() else {
        return blocked(
            node.id(),
            "cannot run because the behavior integration host is unavailable",
        );
    };
    let result = match node.semantics() {
        BehaviorNodeSemantics::MoveTo => host.move_to(&request),
        BehaviorNodeSemantics::PlayAnimation => host.play_animation(&request),
        BehaviorNodeSemantics::ScriptTask => host.script_task(&request),
        _ => return evaluate_task(node),
    };
    BehaviorTreeExecution {
        status: result.status,
        active_node: Some(node.id().to_string()),
        diagnostic: result.diagnostic,
    }
}
