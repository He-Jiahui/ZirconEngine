use zircon_core::CoreError;

use crate::pipeline::manager::EDITOR_ASSET_MANAGER_NAME;
use crate::AssetImportError;

pub(super) fn editor_asset_error(error: AssetImportError) -> CoreError {
    CoreError::Initialization(EDITOR_ASSET_MANAGER_NAME.to_string(), error.to_string())
}
