use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::{Real, Vec3};

use super::{AiAgentTickReport, AiBehaviorTreeDescriptor, AiBlackboardEntry, AiPerceptionSnapshot};

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

/// Spatial perception data sampled by the runtime for editor-only debug overlays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiPerceptionDebugSnapshot {
    pub position: Vec3,
    pub forward: Vec3,
    pub sight_fov_degrees: Real,
    pub sight_range: Real,
    pub hearing_radius: Real,
}

/// Read-only runtime frame delivered to AI editor consumers during play-in-editor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorDebugFrame {
    pub report: AiAgentTickReport,
    pub behavior_tree: Option<String>,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
    pub perception_debug: Option<AiPerceptionDebugSnapshot>,
}

/// Complete debug projection for one runtime world, delivered atomically to editor consumers.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorDebugSnapshot {
    pub world: WorldHandle,
    pub frames: Vec<AiBehaviorDebugFrame>,
}
