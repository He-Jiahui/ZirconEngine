use std::path::Path;

use crate::core::resource::ResourceRegistry;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImportError, AssetImporter};

use super::super::{PackageAssetRegistry, ProjectManifest, ProjectPaths};
use super::ProjectManager;

impl ProjectManager {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let paths = ProjectPaths::from_root(root)?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        paths.ensure_derived_layout()?;
        paths.ensure_asset_roots(&manifest.asset_roots)?;
        let mut package_assets = PackageAssetRegistry::default();
        package_assets.register_project_roots(paths.root(), &manifest.asset_roots)?;
        let asset_registry = AssetRegistryIndex::load_or_rebuild(
            package_assets.project_roots(),
            paths.registry_root(),
        )?;
        Ok(Self {
            paths,
            manifest,
            registry: ResourceRegistry::default(),
            asset_registry,
            package_assets,
            importer: AssetImporter::default(),
            artifact_store: ArtifactStore,
        })
    }
}
