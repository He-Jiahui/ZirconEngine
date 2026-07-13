use std::path::Path;

use crate::asset::AssetImportError;

use super::ProjectManager;

impl ProjectManager {
    pub fn register_package_asset_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        self.package_assets.register_root(package_id, assets_root)
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
        self.package_assets
            .register_package_roots(package_id, asset_roots, package_root)
    }
}
