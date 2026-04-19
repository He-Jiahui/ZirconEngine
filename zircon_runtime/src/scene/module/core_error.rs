use crate::core::CoreError;

use super::LEVEL_MANAGER_NAME;

pub(super) fn scene_core_error(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(LEVEL_MANAGER_NAME.to_string(), message.into())
}
