use std::path::Path;

use crate::asset::AssetImportError;

use super::ProjectManager;

impl ProjectManager {
    pub fn register_package_asset_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        let mut package_assets = self.package_assets.clone();
        package_assets.register_root(package_id, assets_root)?;
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }

    pub fn register_package_asset_roots<Root>(
        &mut self,
        package_id: impl Into<String>,
        asset_roots: impl IntoIterator<Item = Root>,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError>
    where
        Root: AsRef<str>,
    {
        let mut package_assets = self.package_assets.clone();
        package_assets.register_package_roots(package_id, asset_roots, package_root)?;
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }
}
