//! Neutral AI contracts for behavior-tree, blackboard, perception, and agent tick plugins.

mod behavior_tree;
mod blackboard;
mod error;
mod ids;
mod manager;
mod perception;
mod snapshot;
mod tick;

pub use behavior_tree::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameter,
    AiBehaviorNodeParameterValue, AiBehaviorTreeDescriptor, AI_BEHAVIOR_TREE_FORMAT_VERSION,
};
pub use blackboard::{
    AiBlackboardEntry, AiBlackboardKeyDescriptor, AiBlackboardSchemaDescriptor, AiBlackboardValue,
    AiBlackboardValueType,
};
pub use error::AiManagerError;
pub use ids::{AiAgentId, AiBehaviorTreeId, AiBlackboardSchemaId};
pub use manager::AiManager;
pub use perception::{
    AiHearingStimulusEvent, AiHearingStimulusOrigin, AiPerceptionSense, AiPerceptionSnapshot,
    AiPerceptionStimulus,
};
pub use snapshot::{AiAgentRuntimeSnapshot, AiRuntimeSnapshot};
pub use tick::{AiAgentTickReport, AiAgentTickRequest, AiDecisionStatus};
