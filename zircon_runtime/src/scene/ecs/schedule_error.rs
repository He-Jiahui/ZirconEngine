use thiserror::Error;

use crate::scene::ecs::SystemParamError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("system id cannot be empty")]
    EmptySystemId,
    #[error("system set name cannot be empty")]
    EmptySystemSetName,
    #[error("system set {0} must be dot-separated")]
    InvalidSystemSetName(String),
    #[error("system {0} already registered")]
    DuplicateSystem(String),
    #[error("system {0} is still in flight and cannot be retired")]
    SystemInFlight(String),
    #[error("system {0} registration builder was already consumed")]
    SystemBuilderConsumed(String),
    #[error("system {system_id} has invalid tick policy {tick_policy:?} for stage {stage:?}")]
    InvalidTickPolicy {
        system_id: String,
        stage: super::SystemStage,
        tick_policy: super::SceneSystemTickPolicy,
    },
    #[error(
        "system {system_id} may run while virtual time is paused but declares deferred commands"
    )]
    PausedSystemDeferredCommands {
        system_id: String,
        tick_policy: super::SceneSystemTickPolicy,
    },
    #[error(
        "cross-stage ordering constraint in {system_id}: target {target_id} is in {target_stage:?}, not {stage:?}"
    )]
    CrossStageConstraint {
        system_id: String,
        target_id: String,
        stage: super::SystemStage,
        target_stage: super::SystemStage,
    },
    #[error("ordering cycle in {stage:?}: {chain}")]
    OrderingCycle {
        stage: super::SystemStage,
        chain: String,
    },
    #[error("system {system_id} failed to initialize params: {source}")]
    SystemParam {
        system_id: String,
        source: SystemParamError,
    },
    #[error("system {system_id} failed to resolve external access: {message}")]
    ExternalAccess { system_id: String, message: String },
}
