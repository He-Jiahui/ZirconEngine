use zircon_runtime::core::CoreError;

use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

pub(super) fn editor_asset_error(error: impl std::fmt::Display) -> CoreError {
    CoreError::Initialization(EDITOR_ASSET_MANAGER_NAME.to_string(), error.to_string())
}
