//! Neutral AI contracts for behavior-tree, blackboard, perception, and agent tick plugins.
//! 这些 DTO、句柄和服务 trait 是 Runtime、AI 插件与编辑器之间的中立边界；执行状态和调度由实现持有。

#[path = "behavior_tree.rs"]
mod behavior_tree;
#[path = "blackboard.rs"]
mod blackboard;
#[path = "effects.rs"]
mod effects;
#[path = "error.rs"]
mod error;
#[path = "ids.rs"]
mod ids;
#[path = "manager.rs"]
mod manager;
#[path = "perception.rs"]
mod perception;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "tick.rs"]
mod tick;

pub use behavior_tree::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameter,
    AiBehaviorNodeParameterValue, AiBehaviorTreeDescriptor, AI_BEHAVIOR_TREE_FORMAT_VERSION,
};
pub use blackboard::{
    AiBlackboardEntry, AiBlackboardKeyDescriptor, AiBlackboardSchemaDescriptor, AiBlackboardValue,
    AiBlackboardValueType,
};
pub use effects::{
    AiBehaviorEffectCommand, AiBehaviorEffectId, AiBehaviorEffectOutcome, AiBehaviorEffectReceipt,
    AiGameplayEvent,
};
pub use error::AiManagerError;
pub use ids::{AiAgentId, AiBehaviorTreeId, AiBlackboardSchemaId};
pub use manager::AiManager;
pub use perception::{
    AiHearingStimulusEvent, AiHearingStimulusOrigin, AiPerceptionSense, AiPerceptionSnapshot,
    AiPerceptionStimulus,
};
pub use snapshot::{
    AiAgentRuntimeSnapshot, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot,
    AiPerceptionDebugSnapshot, AiRuntimeSnapshot,
};
pub use tick::{AiAgentTickReport, AiAgentTickRequest, AiDecisionStatus, BtNodeResultEvent};
