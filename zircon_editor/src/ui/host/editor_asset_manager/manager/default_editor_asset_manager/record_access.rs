use zircon_runtime::asset::AssetUuid;

use crate::ui::host::editor_asset_manager::AssetCatalogRecord;

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub fn record_by_uuid(&self, asset_uuid: AssetUuid) -> Option<AssetCatalogRecord> {
        self.state
            .read()
            .expect("editor asset state lock poisoned")
            .catalog_by_uuid
            .get(&asset_uuid)
            .cloned()
    }
}
