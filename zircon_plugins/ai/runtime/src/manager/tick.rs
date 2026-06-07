use zircon_runtime::core::framework::ai::{AiAgentTickReport, AiAgentTickRequest, AiManagerError};

use super::execution::evaluate_behavior_tree;
use super::validation::{validate_blackboard_entries, validate_perception_snapshot};
use super::DefaultAiManager;

pub(super) fn tick_agent(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
) -> Result<AiAgentTickReport, AiManagerError> {
    if !request.delta_seconds.is_finite() {
        return Err(AiManagerError::NonFiniteTickDelta);
    }

    let (registered_tree, registered_trees, schema, stored_perception) = {
        let state = manager
            .state
            .lock()
            .expect("AI runtime state mutex poisoned");
        let registered_tree = if let Some(tree_id) = request.behavior_tree {
            let tree = state
                .behavior_trees
                .iter()
                .find(|entry| entry.id == tree_id)
                .ok_or_else(|| AiManagerError::UnknownBehaviorTree { id: tree_id.raw() })?;
            Some(tree.descriptor.clone())
        } else {
            None
        };
        let registered_trees = state
            .behavior_trees
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect::<Vec<_>>();

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
        let stored_perception = state
            .perceptions
            .get(&(request.world, request.entity))
            .cloned();
        (registered_tree, registered_trees, schema, stored_perception)
    };
    validate_blackboard_entries(schema.as_ref(), &request.blackboard)?;

    if let Some(perception) = &request.perception {
        validate_perception_snapshot(request.entity, perception)?;
    }

    let report = if let Some(tree) = &registered_tree {
        let perception = request.perception.as_ref().or(stored_perception.as_ref());
        let execution =
            evaluate_behavior_tree(tree, &registered_trees, &request.blackboard, perception);
        AiAgentTickReport {
            world: request.world,
            entity: request.entity,
            status: execution.status,
            active_node: execution.active_node,
            diagnostic: execution.diagnostic,
        }
    } else {
        AiAgentTickReport::idle(request.world, request.entity)
    };

    let mut state = manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned");
    if let Some(tree) = &registered_tree {
        state
            .active_behavior_trees
            .insert((request.world, request.entity), tree.id.clone());
    } else {
        state
            .active_behavior_trees
            .remove(&(request.world, request.entity));
    }
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
