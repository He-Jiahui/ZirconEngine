use std::collections::HashMap;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiBlackboardEntry,
    AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

#[derive(Clone, Debug, Default)]
pub(super) struct AiRuntimeState {
    pub(super) next_behavior_tree_id: u64,
    pub(super) next_blackboard_schema_id: u64,
    pub(super) behavior_trees: Vec<RegisteredBehaviorTree>,
    pub(super) blackboard_schemas: Vec<RegisteredBlackboardSchema>,
    pub(super) blackboards: HashMap<(WorldHandle, EntityId), Vec<AiBlackboardEntry>>,
    pub(super) perceptions: HashMap<(WorldHandle, EntityId), AiPerceptionSnapshot>,
    pub(super) active_behavior_trees: HashMap<(WorldHandle, EntityId), String>,
    pub(super) last_reports: HashMap<(WorldHandle, EntityId), AiAgentTickReport>,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBehaviorTree {
    pub(super) id: AiBehaviorTreeId,
    pub(super) descriptor: AiBehaviorTreeDescriptor,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBlackboardSchema {
    pub(super) id: AiBlackboardSchemaId,
    pub(super) descriptor: AiBlackboardSchemaDescriptor,
}
