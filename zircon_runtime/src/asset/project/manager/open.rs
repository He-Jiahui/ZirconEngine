//! 项目打开先恢复未完成的持久化代，再建立资产根、注册表和 catalog；Runtime/Editor 调用方应通过此入口获得同一物理项目身份。

use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::core::resource::ResourceRegistry;
use crate::core::runtime::tasks::TaskPool;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImportError, AssetImporter};
use crate::scene::world::SceneComponentSerializerRegistry;

use super::super::{
    PackageAssetRegistry, ProjectCatalogInputGeneration, ProjectManifest, ProjectPaths,
    ResolvedProjectPath,
};
use super::ProjectManager;

impl ProjectManager {
    /// 从项目根建立可导入候选；返回时已恢复磁盘事务并加载注册表，但资源内容仍待扫描导入。
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let paths = ProjectPaths::from_root(root)?;
        Self::open_paths(paths)
    }

    /// Opens a project from a caller-owned physical path without resolving it a second time.
    pub fn open_resolved(root: &ResolvedProjectPath) -> Result<Self, AssetImportError> {
        Self::open_paths(ProjectPaths::from_resolved_root(root))
    }

    fn open_paths(paths: ProjectPaths) -> Result<Self, AssetImportError> {
        let _generation = crate::asset::project::lock_project_generation(paths.root())?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        paths.ensure_derived_layout()?;
        paths.ensure_asset_roots(&manifest.asset_roots)?;
        super::durable_transaction::recover_project_generation(&paths, &manifest)?;
        super::editor_document_transaction::recover_editor_document(&paths, &manifest)?;
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
            scene_component_registry: Arc::new(RwLock::new(
                SceneComponentSerializerRegistry::builtin(),
            )),
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
