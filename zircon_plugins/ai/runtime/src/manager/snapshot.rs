use std::collections::HashSet;

use zircon_runtime::core::framework::ai::{AiAgentRuntimeSnapshot, AiRuntimeSnapshot};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use super::state::AiRuntimeState;
use super::DefaultAiManager;

pub(super) fn runtime_snapshot(manager: &DefaultAiManager) -> AiRuntimeSnapshot {
    let state = manager.lock_state();
    build_runtime_snapshot(&state)
}

fn build_runtime_snapshot(state: &AiRuntimeState) -> AiRuntimeSnapshot {
    let agent_keys = state
        .blackboards
        .keys()
        .chain(state.perceptions.keys())
        .chain(state.active_behavior_trees.keys())
        .chain(state.last_reports.keys())
        .copied()
        .collect::<HashSet<_>>();
    let agents = agent_keys
        .into_iter()
        .filter_map(|key| build_agent_runtime_snapshot(state, key))
        .collect();

    AiRuntimeSnapshot {
        behavior_trees: state
            .behavior_trees
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect(),
        agents,
    }
}

// 调试事件只投影请求的世界与代理，避免每帧克隆完整运行时快照。
pub(super) fn runtime_snapshots_for_agents(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entities: impl IntoIterator<Item = EntityId>,
) -> Vec<AiAgentRuntimeSnapshot> {
    let state = manager.lock_state();
    build_agent_runtime_snapshots(&state, world, entities)
}

fn build_agent_runtime_snapshots(
    state: &AiRuntimeState,
    world: WorldHandle,
    entities: impl IntoIterator<Item = EntityId>,
) -> Vec<AiAgentRuntimeSnapshot> {
    entities
        .into_iter()
        .filter_map(|entity| build_agent_runtime_snapshot(state, (world, entity)))
        .collect()
}

fn build_agent_runtime_snapshot(
    state: &AiRuntimeState,
    (world, entity): (WorldHandle, EntityId),
) -> Option<AiAgentRuntimeSnapshot> {
    let key = (world, entity);
    if !state.blackboards.contains_key(&key)
        && !state.perceptions.contains_key(&key)
        && !state.active_behavior_trees.contains_key(&key)
        && !state.last_reports.contains_key(&key)
    {
        return None;
    }
    Some(AiAgentRuntimeSnapshot {
        world,
        entity,
        behavior_tree: state.active_behavior_trees.get(&key).and_then(|active| {
            state
                .behavior_trees
                .iter()
                .find(|tree| tree.id == active.behavior_tree)
                .map(|tree| tree.descriptor.id.clone())
        }),
        blackboard: state
            .blackboards
            .get(&key)
            .map(|blackboard| blackboard.entries())
            .unwrap_or_default(),
        perception: state.perceptions.get(&key).cloned(),
    })
}

#[cfg(test)]
#[path = "tests/snapshot_optimization_tests.rs"]
mod optimization_tests;
