use zircon_core::CoreError;

use super::PROJECT_ASSET_MANAGER_NAME;

pub(super) fn asset_error(error: impl std::error::Error) -> CoreError {
    CoreError::Initialization(PROJECT_ASSET_MANAGER_NAME.to_string(), error.to_string())
}

pub(super) fn asset_error_message(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(PROJECT_ASSET_MANAGER_NAME.to_string(), message.into())
}
