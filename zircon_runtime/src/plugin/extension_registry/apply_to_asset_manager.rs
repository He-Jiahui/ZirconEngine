use crate::asset::ProjectAssetManager;
use crate::plugin::RuntimeExtensionRegistryError;

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_asset_importers_to_project_asset_manager(
        &self,
        manager: &ProjectAssetManager,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for importer in self.asset_importers().importers() {
            manager
                .register_asset_importer_arc(importer)
                .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()))?;
        }
        Ok(())
    }
}
