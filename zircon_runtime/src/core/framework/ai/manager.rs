use crate::core::framework::scene::{EntityId, WorldHandle};

use super::{
    AiAgentTickReport, AiAgentTickRequest, AiBehaviorTreeDescriptor, AiBehaviorTreeId,
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiManagerError,
    AiPerceptionSnapshot, AiRuntimeSnapshot,
};

pub trait AiManager: Send + Sync {
    fn register_behavior_tree(
        &self,
        descriptor: AiBehaviorTreeDescriptor,
    ) -> Result<AiBehaviorTreeId, AiManagerError>;
    fn behavior_trees(&self) -> Vec<AiBehaviorTreeDescriptor>;
    fn register_blackboard_schema(
        &self,
        descriptor: AiBlackboardSchemaDescriptor,
    ) -> Result<AiBlackboardSchemaId, AiManagerError>;
    fn blackboard_schemas(&self) -> Vec<AiBlackboardSchemaDescriptor>;
    fn set_blackboard_entries(
        &self,
        world: WorldHandle,
        entity: EntityId,
        entries: Vec<AiBlackboardEntry>,
    ) -> Result<(), AiManagerError>;
    fn blackboard_entries(&self, world: WorldHandle, entity: EntityId) -> Vec<AiBlackboardEntry>;
    fn set_perception_snapshot(
        &self,
        world: WorldHandle,
        entity: EntityId,
        snapshot: AiPerceptionSnapshot,
    ) -> Result<(), AiManagerError>;
    fn perception_snapshot(
        &self,
        world: WorldHandle,
        entity: EntityId,
    ) -> Option<AiPerceptionSnapshot>;
    fn tick_agent(&self, request: AiAgentTickRequest) -> Result<AiAgentTickReport, AiManagerError>;
    fn runtime_snapshot(&self) -> AiRuntimeSnapshot;
}
