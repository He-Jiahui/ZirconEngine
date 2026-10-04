use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::{Real, Vec3};

use super::{AiAgentTickReport, AiBehaviorTreeDescriptor, AiBlackboardEntry, AiPerceptionSnapshot};

/// 一个 world/entity 代理的只读聚合；管理器从黑板、感知和活动树填值，最近报告仅影响该代理是否纳入快照。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiAgentRuntimeSnapshot {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub behavior_tree: Option<String>,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 管理器的只读运行态投影，供调试和工具查询；不应用其副本回写代理状态。
pub struct AiRuntimeSnapshot {
    pub behavior_trees: Vec<AiBehaviorTreeDescriptor>,
    pub agents: Vec<AiAgentRuntimeSnapshot>,
}

/// Spatial perception data sampled by the runtime for editor-only debug overlays.
/// 该数据只描述当前感知配置和空间采样，不能作为 AI 输入回写。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiPerceptionDebugSnapshot {
    pub position: Vec3,
    pub forward: Vec3,
    pub sight_fov_degrees: Real,
    pub sight_range: Real,
    pub hearing_radius: Real,
}

/// Read-only runtime frame delivered to AI editor consumers during play-in-editor.
/// 帧把 tick 报告和同一代理的黑板、感知快照配在一起，编辑器按世界事件消费。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorDebugFrame {
    pub report: AiAgentTickReport,
    pub behavior_tree: Option<String>,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
    pub perception_debug: Option<AiPerceptionDebugSnapshot>,
}

/// Complete debug projection for one runtime world, delivered atomically to editor consumers.
/// 注册系统按 world 一次发布完整 frames，编辑器据此清理离开该世界或不再活动的代理。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorDebugSnapshot {
    pub world: WorldHandle,
    pub frames: Vec<AiBehaviorDebugFrame>,
}
