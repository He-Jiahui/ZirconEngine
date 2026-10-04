use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;

use super::{AiBehaviorTreeId, AiBlackboardValue};

/// Identifies one behavior-tree effect independently of its mutable payload.
///
/// The runtime keeps `effect_generation` monotonic for the lifetime of its AI
/// manager. Persisting that generation and the tree instance tick across a
/// process restart belongs to the SaveGame owner.
/// 一次副作用的幂等身份由世界、代理、行为树代次、tick、节点和序号共同组成；
/// 管理器提交阶段据此抑制重放，保存代次的生命周期仍由 SaveGame 负责。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AiBehaviorEffectId {
    pub world: WorldHandle,
    pub entity: u64,
    pub behavior_tree: AiBehaviorTreeId,
    pub compiled_tree_generation: u64,
    pub effect_generation: u64,
    pub tick: u64,
    pub tree_id: String,
    pub node_id: String,
    pub ordinal: u32,
}

/// Typed effects staged by AI nodes and committed by the AI runtime.
/// 执行器先暂存带身份的命令，管理器校验黑板值和事件接收端后才提交；命令本身不直接改 World。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehaviorEffectCommand {
    SetBlackboard {
        effect_id: AiBehaviorEffectId,
        key: String,
        value: AiBlackboardValue,
    },
    EmitEvent {
        effect_id: AiBehaviorEffectId,
        name: String,
        payload: Option<AiBlackboardValue>,
    },
}

/// A typed gameplay event emitted by an AI behavior-tree node.
/// 只有提交成功且已注册事件接收端时才会发布；`effect_id` 让消费方能关联原始节点副作用。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiGameplayEvent {
    pub effect_id: AiBehaviorEffectId,
    pub name: String,
    pub payload: Option<AiBlackboardValue>,
}

/// The committed result of a typed AI behavior effect.
/// 结果区分黑板值是否变化、事件是否入队和重复是否被抑制，调用方不应把 receipt 当作再次执行请求。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehaviorEffectOutcome {
    BlackboardWrite { changed: bool },
    GameplayEventQueued,
    DuplicateSuppressed,
}

/// World event receipt for one committed behavior-tree effect.
/// World 事件通道发布这份回执供测试、调试和上层 gameplay 查询提交结果。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBehaviorEffectReceipt {
    pub effect_id: AiBehaviorEffectId,
    pub outcome: AiBehaviorEffectOutcome,
}
