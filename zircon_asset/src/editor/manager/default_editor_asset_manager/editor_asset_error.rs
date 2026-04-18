use zircon_core::CoreError;

use crate::AssetImportError;

pub(super) fn editor_asset_error(error: AssetImportError) -> CoreError {
    CoreError::Initialization(
        crate::EDITOR_ASSET_MANAGER_NAME.to_string(),
        error.to_string(),
    )
}
