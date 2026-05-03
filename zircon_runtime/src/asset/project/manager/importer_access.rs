use std::sync::Arc;

use crate::asset::{AssetImportError, AssetImporter, AssetImporterHandler, AssetImporterRegistry};

use super::ProjectManager;

impl ProjectManager {
    pub fn importer(&self) -> &AssetImporter {
        &self.importer
    }

    pub fn importer_mut(&mut self) -> &mut AssetImporter {
        &mut self.importer
    }

    pub fn register_asset_importer(
        &mut self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), AssetImportError> {
        self.importer
            .registry_mut()
            .register(importer)
            .map_err(AssetImportError::from)
    }

    pub fn register_asset_importer_arc(
        &mut self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), AssetImportError> {
        self.importer
            .registry_mut()
            .register_arc(importer)
            .map_err(AssetImportError::from)
    }

    pub fn register_asset_importers_from_registry(
        &mut self,
        registry: &AssetImporterRegistry,
    ) -> Result<(), AssetImportError> {
        for importer in registry.importers() {
            self.register_asset_importer_arc(importer)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn register_first_wave_plugin_fixture_importers_for_test(
        &mut self,
    ) -> Result<(), AssetImportError> {
        self.importer
            .register_first_wave_plugin_fixture_importers_for_test()
    }
}
