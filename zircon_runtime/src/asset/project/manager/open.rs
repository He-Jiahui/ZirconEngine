use std::path::Path;
use std::sync::Arc;

use crate::core::resource::ResourceRegistry;
use crate::core::runtime::tasks::TaskPool;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImportError, AssetImporter};

use super::super::{
    PackageAssetRegistry, ProjectCatalogInputGeneration, ProjectManifest, ProjectPaths,
    ResolvedProjectPath,
};
use super::ProjectManager;

impl ProjectManager {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let paths = ProjectPaths::from_root(root)?;
        Self::open_paths(paths)
    }

    /// Opens a project from a caller-owned physical path without resolving it a second time.
    pub fn open_resolved(root: &ResolvedProjectPath) -> Result<Self, AssetImportError> {
        Self::open_paths(ProjectPaths::from_resolved_root(root))
    }

    fn open_paths(paths: ProjectPaths) -> Result<Self, AssetImportError> {
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        paths.ensure_derived_layout()?;
        paths.ensure_asset_roots(&manifest.asset_roots)?;
        super::durable_transaction::recover_project_generation(&paths, &manifest)?;
        let mut package_assets = PackageAssetRegistry::default();
        package_assets.register_project_roots(paths.root(), &manifest.asset_roots)?;
        let asset_registry = AssetRegistryIndex::load_or_rebuild(
            package_assets.project_roots(),
            paths.registry_root(),
        )?;
        let catalog_input_generation = ProjectCatalogInputGeneration::initial(
            paths.root(),
            manifest.clone(),
            package_assets.clone(),
        );
        Ok(Self {
            paths,
            manifest,
            registry: ResourceRegistry::default(),
            asset_registry: Arc::new(asset_registry),
            package_assets,
            catalog_input_generation,
            reference_diagnostics: Arc::new(Default::default()),
            importer: AssetImporter::default(),
            artifact_store: ArtifactStore::default(),
            shader_import_dependencies: Default::default(),
            environment_ibl_parallel_executor: None,
        })
    }

    /// Binds environment IBL staging to the task owner of the active runtime.
    pub(crate) fn set_environment_ibl_parallel_executor(&mut self, executor: TaskPool) {
        self.environment_ibl_parallel_executor = Some(executor);
    }

    #[cfg(test)]
    pub(crate) fn environment_ibl_parallel_executor_for_test(&self) -> Option<&TaskPool> {
        self.environment_ibl_parallel_executor.as_ref()
    }
}
