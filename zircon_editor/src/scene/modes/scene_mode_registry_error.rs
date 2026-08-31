use thiserror::Error;

use crate::core::editor_message::SceneModeId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SceneModeRegistryError {
    #[error("scene mode {mode_id:?} is already registered")]
    DuplicateMode { mode_id: SceneModeId },
    #[error("scene mode {mode_id:?} is not registered")]
    UnknownMode { mode_id: SceneModeId },
    #[error("scene mode {mode_id:?} is already bound to a contribution ticket")]
    ContributionAlreadyOwned { mode_id: SceneModeId },
    #[error(
        "scene mode factory registered as {registered_mode_id:?} produced {produced_mode_id:?}"
    )]
    FactoryModeIdMismatch {
        registered_mode_id: SceneModeId,
        produced_mode_id: SceneModeId,
    },
    #[error("scene mode {mode_id:?} {operation} failed: {message}")]
    CallbackFailure {
        mode_id: SceneModeId,
        operation: &'static str,
        message: String,
    },
}
