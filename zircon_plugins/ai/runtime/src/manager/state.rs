use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiBlackboardEntry,
    AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use crate::behavior_tree::{BehaviorTreeInstanceState, CompiledBehaviorTree};
use crate::blackboard::{BlackboardLayout, BlackboardStore};

#[derive(Debug, Default)]
pub(super) struct AiRuntimeState {
    pub(super) next_behavior_tree_id: u64,
    pub(super) next_blackboard_schema_id: u64,
    pub(super) behavior_trees: Vec<RegisteredBehaviorTree>,
    pub(super) blackboard_schemas: Vec<RegisteredBlackboardSchema>,
    pub(super) blackboards: HashMap<(WorldHandle, EntityId), AgentBlackboard>,
    pub(super) perceptions: HashMap<(WorldHandle, EntityId), AiPerceptionSnapshot>,
    pub(super) active_behavior_trees: HashMap<(WorldHandle, EntityId), ActiveBehaviorAgent>,
    pub(super) behavior_tree_instances: HashMap<(WorldHandle, EntityId), BehaviorTreeInstanceState>,
    pub(super) last_reports: HashMap<(WorldHandle, EntityId), AiAgentTickReport>,
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
