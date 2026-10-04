use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorEffectId, AiBehaviorTreeDescriptor, AiBehaviorTreeId,
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use crate::behavior_tree::{
    reachable_behavior_trees, BehaviorNodeSemantics, BehaviorTreeInstanceState,
    CompiledBehaviorTree,
};
use crate::blackboard::{BlackboardLayout, BlackboardStore};

#[derive(Debug, Default)]
pub(super) struct AiRuntimeState {
    pub(super) next_behavior_tree_id: u64,
    pub(super) next_blackboard_schema_id: u64,
    pub(super) behavior_trees: Vec<RegisteredBehaviorTree>,
    pub(super) compiled_behavior_tree_generation: Arc<[CompiledBehaviorTree]>,
    pub(super) compiled_tree_generation_number: u64,
    pub(super) behavior_effect_sink_requirements: Vec<Option<BehaviorEffectSinkRequirement>>,
    next_effect_generation: u64,
    pub(super) blackboard_schemas: Vec<RegisteredBlackboardSchema>,
    pub(super) blackboards: HashMap<(WorldHandle, EntityId), AgentBlackboard>,
    pub(super) perceptions: HashMap<(WorldHandle, EntityId), AiPerceptionSnapshot>,
    pub(super) active_behavior_trees: HashMap<(WorldHandle, EntityId), ActiveBehaviorAgent>,
    pub(super) behavior_tree_instances: HashMap<(WorldHandle, EntityId), BehaviorTreeInstanceState>,
    pub(super) last_reports: HashMap<(WorldHandle, EntityId), AiAgentTickReport>,
    pub(super) committed_effect_ids: HashSet<AiBehaviorEffectId>,
    pub(super) committed_effect_order: VecDeque<AiBehaviorEffectId>,
}

#[derive(Clone, Debug)]
pub(super) struct BehaviorEffectSinkRequirement {
    pub(super) has_gameplay_events: bool,
    pub(super) first_effect_node: String,
}

impl AiRuntimeState {
    // 注册或撤销后重建不可变树代次；常规 tick 只克隆 Arc 快照。
    pub(super) fn rebuild_compiled_behavior_tree_generation(&mut self) {
        let compiled_generation = self
            .behavior_trees
            .iter()
            .map(|entry| entry.compiled.clone())
            .collect::<Vec<_>>();
        self.behavior_effect_sink_requirements = compiled_generation
            .iter()
            .map(|tree| behavior_effect_sink_requirement(tree, &compiled_generation))
            .collect();
        self.compiled_behavior_tree_generation = compiled_generation.into();
        if let Some(next) = self.compiled_tree_generation_number.checked_add(1) {
            self.compiled_tree_generation_number = next;
        }
    }

    pub(super) fn allocate_effect_generation(&mut self) -> Option<u64> {
        let generation = self.next_effect_generation;
        self.next_effect_generation = generation.checked_add(1)?;
        Some(generation)
    }
}

fn behavior_effect_sink_requirement(
    root: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
) -> Option<BehaviorEffectSinkRequirement> {
    let mut has_gameplay_events = false;
    let mut first_effect_node = None;
    for tree in reachable_behavior_trees(root, registered_trees) {
        for node in tree.nodes() {
            match node.semantics() {
                BehaviorNodeSemantics::SetBlackboard => {
                    first_effect_node.get_or_insert_with(|| node.id().to_string());
                }
                BehaviorNodeSemantics::EmitEvent => {
                    has_gameplay_events = true;
                    first_effect_node.get_or_insert_with(|| node.id().to_string());
                }
                _ => {}
            }
        }
    }
    first_effect_node.map(|first_effect_node| BehaviorEffectSinkRequirement {
        has_gameplay_events,
        first_effect_node,
    })
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ActiveBehaviorAgent {
    pub(super) behavior_tree: AiBehaviorTreeId,
    pub(super) blackboard_schema: Option<AiBlackboardSchemaId>,
    pub(super) pending_delta_seconds: f32,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBehaviorTree {
    pub(super) id: AiBehaviorTreeId,
    pub(super) descriptor: AiBehaviorTreeDescriptor,
    pub(super) compiled: CompiledBehaviorTree,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBlackboardSchema {
    pub(super) id: AiBlackboardSchemaId,
    pub(super) descriptor: AiBlackboardSchemaDescriptor,
    pub(super) layout: Arc<BlackboardLayout>,
}

#[derive(Clone, Debug)]
pub(super) enum AgentBlackboard {
    Dynamic(Vec<AiBlackboardEntry>),
    Dense(BlackboardStore),
}

impl AgentBlackboard {
    pub(super) fn entries(&self) -> Vec<AiBlackboardEntry> {
        match self {
            Self::Dynamic(entries) => entries.clone(),
            Self::Dense(store) => store.entries(),
        }
    }

    pub(super) fn entries_ref(&self) -> &[AiBlackboardEntry] {
        match self {
            Self::Dynamic(entries) => entries,
            Self::Dense(store) => store.entries_ref(),
        }
    }
}

#[cfg(test)]
#[path = "tests/state_optimization_tests.rs"]
mod optimization_tests;
