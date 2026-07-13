use crate::asset::registry::AssetRegistryIndex;
use crate::core::resource::ResourceRegistry;

use super::super::{PackageAssetRegistry, ProjectManifest, ProjectPaths};
use super::ProjectManager;
use crate::asset::AssetImportError;
use std::path::Path;

impl ProjectManager {
    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn registry(&self) -> &ResourceRegistry {
        &self.registry
    }

    pub fn asset_registry(&self) -> &AssetRegistryIndex {
        &self.asset_registry
    }

    pub fn package_assets(&self) -> &PackageAssetRegistry {
        &self.package_assets
    }

    pub fn project_asset_roots(&self) -> &[std::path::PathBuf] {
        self.package_assets.project_roots()
    }

    pub(super) fn registry_scan_roots(&self) -> Vec<std::path::PathBuf> {
        self.package_assets
            .project_roots()
            .iter()
            .cloned()
            .chain(
                self.package_assets
                    .iter()
                    .map(|(_, root)| root.to_path_buf()),
            )
            .collect()
    }

    pub fn primary_project_asset_root(&self) -> Result<&Path, AssetImportError> {
        self.package_assets.primary_project_root()
    }

    pub fn project_asset_root_for_source_path(
        &self,
        source_path: &Path,
    ) -> Result<&Path, AssetImportError> {
        let roots = self
            .project_asset_roots()
            .iter()
            .filter(|root| source_path.starts_with(root))
            .collect::<Vec<_>>();
        match roots.as_slice() {
            [root] => Ok(root.as_path()),
            [] => Err(AssetImportError::SourceOutsideProjectAssetRoots {
                path: source_path.to_path_buf(),
            }),
            _ => Err(AssetImportError::AmbiguousProjectSourcePath {
                path: source_path.to_path_buf(),
                roots: roots.into_iter().cloned().collect(),
            }),
        }
    }
}
