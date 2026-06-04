use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiAgentTickRequest, AiBehaviorTreeDescriptor, AiBehaviorTreeId,
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiManager,
    AiManagerError, AiPerceptionSnapshot, AiRuntimeSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use super::{behavior_tree, blackboard, perception, snapshot, tick, DefaultAiManager};

impl AiManager for DefaultAiManager {
    fn register_behavior_tree(
        &self,
        descriptor: AiBehaviorTreeDescriptor,
    ) -> Result<AiBehaviorTreeId, AiManagerError> {
        behavior_tree::register(self, descriptor)
    }

    fn behavior_trees(&self) -> Vec<AiBehaviorTreeDescriptor> {
        behavior_tree::descriptors(self)
    }

    fn register_blackboard_schema(
        &self,
        descriptor: AiBlackboardSchemaDescriptor,
    ) -> Result<AiBlackboardSchemaId, AiManagerError> {
        blackboard::register_schema(self, descriptor)
    }

    fn blackboard_schemas(&self) -> Vec<AiBlackboardSchemaDescriptor> {
        blackboard::schemas(self)
    }

    fn set_blackboard_entries(
        &self,
        world: WorldHandle,
        entity: EntityId,
        entries: Vec<AiBlackboardEntry>,
    ) -> Result<(), AiManagerError> {
        blackboard::set_entries(self, world, entity, entries)
    }

    fn blackboard_entries(&self, world: WorldHandle, entity: EntityId) -> Vec<AiBlackboardEntry> {
        blackboard::entries(self, world, entity)
    }

    fn set_perception_snapshot(
        &self,
        world: WorldHandle,
        entity: EntityId,
        snapshot: AiPerceptionSnapshot,
    ) -> Result<(), AiManagerError> {
        perception::set_snapshot(self, world, entity, snapshot)
    }

    fn perception_snapshot(
        &self,
        world: WorldHandle,
        entity: EntityId,
    ) -> Option<AiPerceptionSnapshot> {
        perception::snapshot(self, world, entity)
    }

    fn tick_agent(&self, request: AiAgentTickRequest) -> Result<AiAgentTickReport, AiManagerError> {
        tick::tick_agent(self, request)
    }

    fn runtime_snapshot(&self) -> AiRuntimeSnapshot {
        snapshot::runtime_snapshot(self)
    }
}
