use zircon_core::CoreError;

use crate::pipeline::manager::EDITOR_ASSET_MANAGER_NAME;
use crate::AssetUuid;

pub(super) fn parse_uuid(uuid: &str) -> Result<AssetUuid, CoreError> {
    uuid.parse::<AssetUuid>().map_err(|error| {
        CoreError::Initialization(
            EDITOR_ASSET_MANAGER_NAME.to_string(),
            format!("invalid asset uuid {uuid}: {error}"),
        )
    })
}
