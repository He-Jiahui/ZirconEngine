use std::collections::HashMap;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiBlackboardEntry,
    AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use crate::behavior_tree::{BehaviorTreeInstanceState, CompiledBehaviorTree};

#[derive(Debug, Default)]
pub(super) struct AiRuntimeState {
    pub(super) next_behavior_tree_id: u64,
    pub(super) next_blackboard_schema_id: u64,
    pub(super) behavior_trees: Vec<RegisteredBehaviorTree>,
    pub(super) blackboard_schemas: Vec<RegisteredBlackboardSchema>,
    pub(super) blackboards: HashMap<(WorldHandle, EntityId), Vec<AiBlackboardEntry>>,
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
}
