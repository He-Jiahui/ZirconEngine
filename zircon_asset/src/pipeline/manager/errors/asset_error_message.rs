use zircon_core::CoreError;

use super::super::PROJECT_ASSET_MANAGER_NAME;

pub(in crate::pipeline::manager) fn asset_error_message(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(PROJECT_ASSET_MANAGER_NAME.to_string(), message.into())
}
