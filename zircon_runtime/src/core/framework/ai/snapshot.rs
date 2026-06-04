use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};

use super::{AiBehaviorTreeDescriptor, AiBlackboardEntry, AiPerceptionSnapshot};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiAgentRuntimeSnapshot {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub behavior_tree: Option<String>,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiRuntimeSnapshot {
    pub behavior_trees: Vec<AiBehaviorTreeDescriptor>,
    pub agents: Vec<AiAgentRuntimeSnapshot>,
}
