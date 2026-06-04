use std::collections::HashSet;

use zircon_runtime::core::framework::ai::{AiAgentRuntimeSnapshot, AiRuntimeSnapshot};

use super::state::AiRuntimeState;
use super::DefaultAiManager;

pub(super) fn runtime_snapshot(manager: &DefaultAiManager) -> AiRuntimeSnapshot {
    let state = manager
        .state
        .lock()
        .expect("AI runtime state mutex poisoned");
    build_runtime_snapshot(&state)
}

fn build_runtime_snapshot(state: &AiRuntimeState) -> AiRuntimeSnapshot {
    let agent_keys = state
        .blackboards
        .keys()
        .chain(state.perceptions.keys())
        .chain(state.active_behavior_trees.keys())
        .copied()
        .collect::<HashSet<_>>();
    let agents = agent_keys
        .into_iter()
        .map(|(world, entity)| AiAgentRuntimeSnapshot {
            world,
            entity,
            behavior_tree: state.active_behavior_trees.get(&(world, entity)).cloned(),
            blackboard: state
                .blackboards
                .get(&(world, entity))
                .cloned()
                .unwrap_or_default(),
            perception: state.perceptions.get(&(world, entity)).cloned(),
        })
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
