use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

use super::{AiBehaviorTreeId, AiBlackboardEntry, AiBlackboardSchemaId, AiPerceptionSnapshot};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiAgentTickRequest {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub behavior_tree: Option<AiBehaviorTreeId>,
    pub blackboard_schema: Option<AiBlackboardSchemaId>,
    pub delta_seconds: Real,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDecisionStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiAgentTickReport {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub status: AiDecisionStatus,
    pub active_node: Option<String>,
    pub diagnostic: Option<String>,
}

impl AiAgentTickReport {
    pub fn idle(world: WorldHandle, entity: EntityId) -> Self {
        Self {
            world,
            entity,
            status: AiDecisionStatus::Idle,
            active_node: None,
            diagnostic: None,
        }
    }
}
