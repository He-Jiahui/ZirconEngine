use std::path::Path;

use crate::asset::AssetImportError;
use crate::plugin::PluginPackageManifest;

use super::ProjectManager;

impl ProjectManager {
    pub fn register_package_asset_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        self.package_assets.register_root(package_id, assets_root)
    }

    pub fn register_package_manifest_asset_roots(
        &mut self,
        manifest: &PluginPackageManifest,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        self.package_assets
            .register_manifest_roots(manifest, package_root)
    }
}
