use thiserror::Error;

use crate::core::editor_message::SceneModeId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SceneModeStackError {
    #[error(
        "scene mode activation `{}` does not match instantiated mode `{}`",
        activation_mode_id.as_str(),
        mode_id.as_str()
    )]
    ActivationModeIdMismatch {
        activation_mode_id: SceneModeId,
        mode_id: SceneModeId,
    },
    #[error("scene mode activation cannot use reserved built-in id `{}`", mode_id.as_str())]
    ReservedBuiltinActivation { mode_id: SceneModeId },
    #[error("built-in scene mode `{}` cannot be used as an overlay", mode_id.as_str())]
    BuiltInOverlay { mode_id: SceneModeId },
    #[error("scene mode `{}` is already active", mode_id.as_str())]
    DuplicateMode { mode_id: SceneModeId },
    #[error(
        "scene mode `{}` failed to enter: {replacement_message}; restoring `{}` also failed: {rollback_message}",
        replacement_mode_id.as_str(),
        rollback_mode_id.as_str()
    )]
    BaseReplacementRollbackFailure {
        replacement_mode_id: SceneModeId,
        replacement_message: String,
        rollback_mode_id: SceneModeId,
        rollback_message: String,
    },
    #[error("scene mode `{}` failed to enter: {message}", mode_id.as_str())]
    EnterFailure {
        mode_id: SceneModeId,
        message: String,
    },
}

impl SceneModeStackError {
    pub fn mode_id(&self) -> &SceneModeId {
        match self {
            Self::BaseReplacementRollbackFailure {
                replacement_mode_id,
                ..
            } => replacement_mode_id,
            Self::ActivationModeIdMismatch { mode_id, .. }
            | Self::BuiltInOverlay { mode_id }
            | Self::DuplicateMode { mode_id }
            | Self::EnterFailure { mode_id, .. }
            | Self::ReservedBuiltinActivation { mode_id } => mode_id,
        }
    }
}
