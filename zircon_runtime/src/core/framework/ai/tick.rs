use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

use super::{AiBehaviorTreeId, AiBlackboardEntry, AiBlackboardSchemaId, AiPerceptionSnapshot};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 一次代理决策的输入边界。树与布局句柄必须已注册；黑板和感知按本次请求提供，执行状态由管理器保留。
pub struct AiAgentTickRequest {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub behavior_tree: Option<AiBehaviorTreeId>,
    pub blackboard_schema: Option<AiBlackboardSchemaId>,
    pub delta_seconds: Real,
    pub blackboard: Vec<AiBlackboardEntry>,
    pub perception: Option<AiPerceptionSnapshot>,
}

/// 行为树执行器向管理器和编辑器报告空闲、运行、成功、失败或阻塞状态。
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
/// 决策结果与当前活动节点的轻量投影；编辑器事件可由 node_result_event 派生，但无活动节点时没有该事件。
pub struct AiAgentTickReport {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub status: AiDecisionStatus,
    pub active_node: Option<String>,
    pub diagnostic: Option<String>,
}

/// Typed node-state update consumed by read-only behavior-tree editor mirrors.
/// 它由 tick 报告按活动节点派生并发送到 World 事件通道；没有活动节点时不会生成事件。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BtNodeResultEvent {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub node_id: String,
    pub status: AiDecisionStatus,
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

    pub fn node_result_event(&self) -> Option<BtNodeResultEvent> {
        Some(BtNodeResultEvent {
            world: self.world.clone(),
            entity: self.entity,
            node_id: self.active_node.clone()?,
            status: self.status.clone(),
            diagnostic: self.diagnostic.clone(),
        })
    }
}
