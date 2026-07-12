use zircon_runtime::core::framework::ai::{AiAgentTickReport, AiAgentTickRequest, AiManagerError};
use zircon_runtime::core::framework::scene::WorldHandle;

use super::state::ActiveBehaviorAgent;
use super::validation::{validate_blackboard_entries, validate_perception_snapshot};
use super::DefaultAiManager;
use crate::behavior_tree::{evaluate_behavior_tree, BehaviorTreeInstanceState};
use crate::AiBehaviorTickLod;

pub(super) fn tick_agent(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
) -> Result<AiAgentTickReport, AiManagerError> {
    if !request.delta_seconds.is_finite() {
        return Err(AiManagerError::NonFiniteTickDelta);
    }

    let agent_key = (request.world, request.entity);
    let (
        execution_lease,
        registered_tree,
        registered_trees,
        schema,
        stored_perception,
        mut instance,
    ) = {
        let mut state = manager
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let registered_tree = if let Some(tree_id) = request.behavior_tree {
            let tree = state
                .behavior_trees
                .iter()
                .find(|entry| entry.id == tree_id)
                .ok_or_else(|| AiManagerError::UnknownBehaviorTree { id: tree_id.raw() })?;
            Some(tree.clone())
        } else {
            None
        };
        let registered_trees = state
            .behavior_trees
            .iter()
            .map(|entry| entry.compiled.clone())
            .collect::<Vec<_>>();
        let implementation_slots = registered_trees
            .iter()
            .flat_map(|tree| tree.implementation_slots())
            .collect::<Vec<_>>();
        let catalog = manager
            .behavior_node_catalog
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owners = implementation_slots
            .into_iter()
            .map(|slot| {
                catalog
                    .owner_for_slot(slot)
                    .ok_or_else(|| AiManagerError::UnknownBehaviorTree {
                        id: request.behavior_tree.map(|id| id.raw()).unwrap_or_default(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        drop(catalog);
        let execution_lease = manager
            .behavior_node_execution_gate
            .acquire(owners)
            .ok_or_else(|| AiManagerError::UnknownBehaviorTree {
                id: request.behavior_tree.map(|id| id.raw()).unwrap_or_default(),
            })?;

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
        let instance = state
            .behavior_tree_instances
            .remove(&agent_key)
            .unwrap_or_else(BehaviorTreeInstanceState::default);
        (
            execution_lease,
            registered_tree,
            registered_trees,
            schema,
            stored_perception,
            instance,
        )
    };
    validate_blackboard_entries(schema.as_ref(), &request.blackboard)?;

    if let Some(perception) = &request.perception {
        validate_perception_snapshot(request.entity, perception)?;
    }

    let report = if let Some(tree) = &registered_tree {
        let perception = request.perception.as_ref().or(stored_perception.as_ref());
        let execution = evaluate_behavior_tree(
            &tree.compiled,
            &registered_trees,
            &request.blackboard,
            perception,
            request.delta_seconds,
            &mut instance,
        );
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
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(tree) = &registered_tree {
        state.active_behavior_trees.insert(
            agent_key,
            ActiveBehaviorAgent {
                behavior_tree: tree.id,
                blackboard_schema: request.blackboard_schema,
                pending_delta_seconds: 0.0,
            },
        );
        state.behavior_tree_instances.insert(agent_key, instance);
    } else {
        state.active_behavior_trees.remove(&agent_key);
        state.behavior_tree_instances.remove(&agent_key);
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
    drop(state);
    drop(execution_lease);
    Ok(report)
}

pub(super) fn tick_active_agents(
    manager: &DefaultAiManager,
    world: WorldHandle,
    delta_seconds: f32,
) -> Result<Vec<AiAgentTickReport>, AiManagerError> {
    tick_active_agents_with_lod(manager, world, delta_seconds, 0, |_| {
        AiBehaviorTickLod::Full
    })
}

pub(super) fn active_agent_entities(manager: &DefaultAiManager, world: WorldHandle) -> Vec<u64> {
    manager
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .active_behavior_trees
        .keys()
        .filter_map(|(agent_world, entity)| (*agent_world == world).then_some(*entity))
        .collect()
}

pub(super) fn tick_active_agents_with_lod(
    manager: &DefaultAiManager,
    world: WorldHandle,
    delta_seconds: f32,
    frame: u64,
    mut lod_for_entity: impl FnMut(u64) -> AiBehaviorTickLod,
) -> Result<Vec<AiAgentTickReport>, AiManagerError> {
    if !delta_seconds.is_finite() {
        return Err(AiManagerError::NonFiniteTickDelta);
    }
    let requests = {
        let mut state = manager
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let candidates = state
            .active_behavior_trees
            .iter()
            .filter(|((agent_world, _), _)| *agent_world == world)
            .map(|((_, entity), active)| (*entity, active.behavior_tree, active.blackboard_schema))
            .collect::<Vec<_>>();
        let mut requests = Vec::new();
        for (entity, behavior_tree, blackboard_schema) in candidates {
            let active = state
                .active_behavior_trees
                .get_mut(&(world, entity))
                .ok_or(AiManagerError::UnknownBehaviorTree {
                    id: behavior_tree.raw(),
                })?;
            active.pending_delta_seconds += delta_seconds.max(0.0);
            if !lod_for_entity(entity).should_tick(frame, entity) {
                continue;
            }
            let elapsed = std::mem::take(&mut active.pending_delta_seconds);
            requests.push(AiAgentTickRequest {
                world,
                entity,
                behavior_tree: Some(behavior_tree),
                blackboard_schema,
                delta_seconds: elapsed,
                blackboard: state
                    .blackboards
                    .get(&(world, entity))
                    .cloned()
                    .unwrap_or_default(),
                perception: state.perceptions.get(&(world, entity)).cloned(),
            });
        }
        requests
    };
    requests
        .into_iter()
        .map(|request| tick_agent(manager, request))
        .collect()
}
