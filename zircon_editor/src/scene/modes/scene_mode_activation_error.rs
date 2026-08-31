use thiserror::Error;

use crate::core::editor_message::SceneModeId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum SceneModeActivationError {
    #[error("custom scene mode activation cannot use reserved built-in id {mode_id:?}")]
    ReservedBuiltInId { mode_id: SceneModeId },
}
