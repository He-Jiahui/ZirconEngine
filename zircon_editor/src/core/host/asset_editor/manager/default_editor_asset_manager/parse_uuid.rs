use zircon_runtime::core::CoreError;

use zircon_runtime::asset::AssetUuid;

use crate::core::host::module::EDITOR_ASSET_MANAGER_NAME;

pub(super) fn parse_uuid(uuid: &str) -> Result<AssetUuid, CoreError> {
    uuid.parse::<AssetUuid>().map_err(|error| {
        CoreError::Initialization(
            EDITOR_ASSET_MANAGER_NAME.to_string(),
            format!("invalid asset uuid {uuid}: {error}"),
        )
    })
}
