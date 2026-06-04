use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiAgentTickRequest, AiDecisionStatus, AiManagerError,
};

use super::validation::{validate_blackboard_entries, validate_perception_snapshot};
use super::DefaultAiManager;

pub(super) fn tick_agent(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
) -> Result<AiAgentTickReport, AiManagerError> {
    if !request.delta_seconds.is_finite() {
        return Err(AiManagerError::NonFiniteTickDelta);
    }

    let mut state = manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned");
    let registered_tree = if let Some(tree_id) = request.behavior_tree {
        let tree = state
            .behavior_trees
            .iter()
            .find(|entry| entry.id == tree_id)
            .ok_or_else(|| AiManagerError::UnknownBehaviorTree { id: tree_id.raw() })?;
        Some((
            tree.descriptor.id.clone(),
            tree.descriptor.root_node.clone(),
        ))
    } else {
        None
    };

    let schema = if let Some(schema_id) = request.blackboard_schema {
        let schema = state
            .blackboard_schemas
            .iter()
            .find(|entry| entry.id == schema_id)
            .ok_or_else(|| AiManagerError::UnknownBlackboardSchema {
                id: schema_id.raw(),
            })?;
        Some(schema.descriptor.clone())
    } else {
        None
    };
    validate_blackboard_entries(schema.as_ref(), &request.blackboard)?;

    if let Some(perception) = &request.perception {
        validate_perception_snapshot(request.entity, perception)?;
    }

    let report = if let Some((tree_name, root_node)) = registered_tree {
        state
            .active_behavior_trees
            .insert((request.world, request.entity), tree_name);
        AiAgentTickReport {
            world: request.world,
            entity: request.entity,
            status: AiDecisionStatus::Blocked,
            active_node: Some(root_node),
            diagnostic: Some(
                "AI behavior-tree execution is registered but not promoted yet".to_string(),
            ),
        }
    } else {
        state
            .active_behavior_trees
            .remove(&(request.world, request.entity));
        AiAgentTickReport::idle(request.world, request.entity)
    };

    state
        .blackboards
        .insert((request.world, request.entity), request.blackboard.clone());
    if let Some(perception) = request.perception.clone() {
        state
            .perceptions
            .insert((request.world, request.entity), perception);
    }
    state
        .last_reports
        .insert((request.world, request.entity), report.clone());
    Ok(report)
}
