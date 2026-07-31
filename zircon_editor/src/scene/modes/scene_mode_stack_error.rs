use thiserror::Error;

use crate::core::editor_message::SceneModeId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SceneModeStackError {
    #[error("scene mode `{}` is already active", mode_id.as_str())]
    DuplicateMode { mode_id: SceneModeId },
    #[error("scene mode `{}` failed to enter: {message}", mode_id.as_str())]
    EnterFailure {
        mode_id: SceneModeId,
        message: String,
    },
}

impl SceneModeStackError {
    pub fn mode_id(&self) -> &SceneModeId {
        match self {
            Self::DuplicateMode { mode_id } | Self::EnterFailure { mode_id, .. } => mode_id,
        }
    }
}
