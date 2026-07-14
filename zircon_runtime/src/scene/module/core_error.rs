use crate::core::manager::LEVEL_MANAGER_NAME;
use crate::core::CoreError;

pub(super) fn scene_core_error(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(LEVEL_MANAGER_NAME.to_string(), message.into())
}
