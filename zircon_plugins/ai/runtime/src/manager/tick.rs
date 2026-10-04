use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiAgentTickRequest, AiBehaviorEffectCommand, AiDecisionStatus,
    AiManagerError,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use super::state::{ActiveBehaviorAgent, AgentBlackboard};
use super::validation::{validate_blackboard_entries, validate_perception_snapshot};
use super::DefaultAiManager;
use crate::behavior_tree::{
    abort_behavior_tree_instance, evaluate_behavior_tree, BehaviorIntegrationHost,
    BehaviorTreeInstanceState,
};
use crate::blackboard::BlackboardStore;
use crate::AiBehaviorTickLod;

pub(super) fn tick_agent(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
) -> Result<AiAgentTickReport, AiManagerError> {
    tick_agent_with_source(manager, request, false, None)
}

#[cfg(test)]
pub(super) fn tick_agent_with_integration_host(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
    integration_host: &mut dyn BehaviorIntegrationHost,
) -> Result<AiAgentTickReport, AiManagerError> {
    tick_agent_with_source(manager, request, false, Some(integration_host))
}

fn tick_agent_with_source(
    manager: &DefaultAiManager,
    request: AiAgentTickRequest,
    use_stored_blackboard: bool,
    mut integration_host: Option<&mut dyn BehaviorIntegrationHost>,
) -> Result<AiAgentTickReport, AiManagerError> {
    if !request.delta_seconds.is_finite() {
        return Err(AiManagerError::NonFiniteTickDelta);
    }
    if let Some(perception) = &request.perception {
        validate_perception_snapshot(request.entity, perception)?;
    }

    let agent_key = (request.world, request.entity);
    let (
        execution_lease,
        registered_tree,
        registered_trees,
        schema,
        stored_blackboard,
        stored_perception,
        compiled_tree_generation,
        effect_generation,
        effect_sink_requirement,
        mut instance,
    ) = {
        let mut state = manager
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let registered_tree = if let Some(tree_id) = request.behavior_tree {
            let (tree_index, tree) = state
                .behavior_trees
                .iter()
                .enumerate()
                .find(|(_, entry)| entry.id == tree_id)
                .ok_or_else(|| AiManagerError::UnknownBehaviorTree { id: tree_id.raw() })?;
            Some((tree.id, tree_index))
        } else {
            None
        };
        let registered_trees = Arc::clone(&state.compiled_behavior_tree_generation);
        let compiled_tree_generation = state.compiled_tree_generation_number;
        let effect_generation = registered_tree
            .as_ref()
            .and_then(|_| state.allocate_effect_generation());
        let effect_sink_requirement = registered_tree.as_ref().and_then(|(_, tree_index)| {
            state
                .behavior_effect_sink_requirements
                .get(*tree_index)
                .cloned()
                .flatten()
        });
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
            Some(schema.clone())
        } else {
            None
        };
        if schema.is_none()
            && registered_tree.as_ref().is_some_and(|(_, tree_index)| {
                registered_trees[*tree_index].reachable_tree_has_abort_observers(&registered_trees)
            })
        {
            return Err(AiManagerError::BehaviorObserverRequiresBlackboardSchema {
                tree_id: registered_tree
                    .as_ref()
                    .map(|(_, tree_index)| registered_trees[*tree_index].id().to_string())
                    .unwrap_or_default(),
            });
        }
        let validation_entries = if use_stored_blackboard {
            state
                .blackboards
                .get(&agent_key)
                .map(AgentBlackboard::entries_ref)
                .unwrap_or_default()
        } else {
            &request.blackboard
        };
        validate_blackboard_entries(
            schema.as_ref().map(|schema| &schema.descriptor),
            validation_entries,
        )?;
        let stored_perception = state
            .perceptions
            .get(&(request.world, request.entity))
            .cloned();
        // 取出代理私有状态后释放总锁，节点执行可回调宿主；执行错误路径须放回状态。
        let stored_blackboard = state.blackboards.remove(&agent_key);
        let instance = state
            .behavior_tree_instances
            .remove(&agent_key)
            .unwrap_or_else(BehaviorTreeInstanceState::default);
        (
            execution_lease,
            registered_tree,
            registered_trees,
            schema,
            stored_blackboard,
            stored_perception,
            compiled_tree_generation,
            effect_generation,
            effect_sink_requirement,
            instance,
        )
    };
    let (mut stored_blackboard, changed_slots) = if let Some(schema) = &schema {
        let mut store = match stored_blackboard {
            Some(AgentBlackboard::Dense(store))
                if store.layout().schema_id() == schema.layout.schema_id() =>
            {
                store
            }
            previous => {
                let mut store = BlackboardStore::new(schema.layout.clone());
                if use_stored_blackboard {
                    let entries = previous
                        .as_ref()
                        .map(AgentBlackboard::entries)
                        .unwrap_or_default();
                    store.synchronize(&entries).map_err(|error| {
                        super::blackboard::map_runtime_error(schema.layout.schema_id(), error)
                    })?;
                }
                store
            }
        };
        if !use_stored_blackboard {
            store.synchronize(&request.blackboard).map_err(|error| {
                super::blackboard::map_runtime_error(schema.layout.schema_id(), error)
            })?;
        }
        let changed_slots = store.drain_changed_slots();
        (AgentBlackboard::Dense(store), changed_slots)
    } else if use_stored_blackboard {
        (
            stored_blackboard.unwrap_or_else(|| AgentBlackboard::Dynamic(Vec::new())),
            Vec::new(),
        )
    } else {
        (
            AgentBlackboard::Dynamic(request.blackboard.clone()),
            Vec::new(),
        )
    };
    let effect_sink_failure = effect_sink_requirement.and_then(|requirement| {
        let result = integration_host
            .as_deref()
            .ok_or_else(|| "AI behavior effect integration host is unavailable".to_string())
            .and_then(|host| {
                host.can_publish_behavior_effects(requirement.has_gameplay_events, true)
            });
        result
            .err()
            .map(|error| (requirement.first_effect_node, error))
    });

    let report = if let Some((behavior_tree_id, tree_index)) = registered_tree {
        let tree = &registered_trees[tree_index];
        if let Some((node_id, error)) = effect_sink_failure {
            AiAgentTickReport {
                world: request.world,
                entity: request.entity,
                status: AiDecisionStatus::Blocked,
                active_node: Some(node_id),
                diagnostic: Some(format!("AI behavior effect sink is unavailable: {error}")),
            }
        } else {
            let blackboard = stored_blackboard.entries_ref();
            let blackboard_store = match &stored_blackboard {
                AgentBlackboard::Dense(store) => Some(store),
                AgentBlackboard::Dynamic(_) => None,
            };
            let perception = request.perception.as_ref().or(stored_perception.as_ref());
            let (mut execution, effect_commands) = match evaluate_behavior_tree(
                tree,
                &registered_trees,
                request.world,
                behavior_tree_id,
                compiled_tree_generation,
                effect_generation,
                blackboard,
                perception,
                request.delta_seconds,
                schema.as_ref().map(|schema| schema.layout.as_ref()),
                blackboard_store,
                &changed_slots,
                &mut instance,
                request.entity,
                match integration_host.as_mut() {
                    Some(host) => Some(&mut **host),
                    None => None,
                },
            ) {
                Ok(execution) => execution,
                Err(error) => {
                    let mut state = manager.lock_state();
                    state.blackboards.insert(agent_key, stored_blackboard);
                    state.behavior_tree_instances.insert(agent_key, instance);
                    drop(state);
                    drop(execution_lease);
                    return Err(error);
                }
            };
            if !effect_commands.is_empty() {
                let active_effect_node = effect_commands
                    .first()
                    .map(|command| behavior_effect_node_id(command).to_string());
                if let Err(error) = super::effects::commit_behavior_effects(
                    manager,
                    effect_commands,
                    &mut stored_blackboard,
                    integration_host,
                ) {
                    execution.status = AiDecisionStatus::Blocked;
                    execution.active_node = active_effect_node;
                    execution.diagnostic = Some(format!(
                        "AI behavior effect batch could not commit: {error}"
                    ));
                }
            }
            AiAgentTickReport {
                world: request.world,
                entity: request.entity,
                status: execution.status,
                active_node: execution.active_node,
                diagnostic: execution.diagnostic,
            }
        }
    } else {
        let blackboard = stored_blackboard.entries_ref();
        abort_behavior_tree_instance(
            &registered_trees,
            blackboard,
            request.perception.as_ref().or(stored_perception.as_ref()),
            request.delta_seconds,
            &mut instance,
            request.entity,
            integration_host,
        );
        AiAgentTickReport::idle(request.world, request.entity)
    };

    let mut state = manager
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((tree_id, _)) = registered_tree {
        state.active_behavior_trees.insert(
            agent_key,
            ActiveBehaviorAgent {
                behavior_tree: tree_id,
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
        .insert((request.world, request.entity), stored_blackboard);
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

fn behavior_effect_node_id(command: &AiBehaviorEffectCommand) -> &str {
    match command {
        AiBehaviorEffectCommand::SetBlackboard { effect_id, .. }
        | AiBehaviorEffectCommand::EmitEvent { effect_id, .. } => &effect_id.node_id,
    }
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
    tick_active_agents_with_lod_inner(
        manager,
        world,
        delta_seconds,
        frame,
        &mut lod_for_entity,
        None,
    )
}

pub(super) fn tick_active_agents_with_lod_and_integration_host(
    manager: &DefaultAiManager,
    world: WorldHandle,
    delta_seconds: f32,
    frame: u64,
    mut lod_for_entity: impl FnMut(u64) -> AiBehaviorTickLod,
    integration_host: &mut dyn BehaviorIntegrationHost,
) -> Result<Vec<AiAgentTickReport>, AiManagerError> {
    tick_active_agents_with_lod_inner(
        manager,
        world,
        delta_seconds,
        frame,
        &mut lod_for_entity,
        Some(integration_host),
    )
}

fn tick_active_agents_with_lod_inner(
    manager: &DefaultAiManager,
    world: WorldHandle,
    delta_seconds: f32,
    frame: u64,
    lod_for_entity: &mut dyn FnMut(u64) -> AiBehaviorTickLod,
    mut integration_host: Option<&mut dyn BehaviorIntegrationHost>,
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
            // 降频帧累积经过的时间；实际执行时一次性传给计时节点并清零。
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
                blackboard: Vec::new(),
                perception: state.perceptions.get(&(world, entity)).cloned(),
            });
        }
        requests
    };
    let mut reports = Vec::with_capacity(requests.len());
    // BUG: [CR-AI-MANAGER-0001] 前面已清零全部入选代理的累计时长；任一 tick 报错会使后续请求丢失该时长。
    for request in requests {
        let report = if let Some(host) = integration_host.as_mut() {
            tick_agent_with_source(manager, request, true, Some(&mut **host))?
        } else {
            tick_agent_with_source(manager, request, true, None)?
        };
        reports.push(report);
    }
    Ok(reports)
}
