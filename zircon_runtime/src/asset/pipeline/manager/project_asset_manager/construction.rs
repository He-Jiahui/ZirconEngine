use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::resource::{ResourceManager, RuntimeResourceState};
use crate::core::CoreError;

use super::super::builtins::resource_manager_with_builtins;
use super::super::errors::asset_error_message;
use super::project_asset_manager::{ProjectSourcePathIndex, ProjectWatcherActivation};
use super::ProjectAssetManager;
use crate::asset::artifact::IblBakeArtifactCacheStore;
use crate::asset::project::ProjectManager;
use crate::asset::worker_pool::{
    AssetWorkerPool, AssetWorkerPoolFrameSampler, AssetWorkerPoolOptions,
    AssetWorkerThreadBudgetSource,
};
use crate::asset::{
    AssetId, AssetImportError, AssetImporter, AssetImporterCapabilityReport, AssetImporterHandler,
    AssetImporterRegistry, AssetUri, ShaderAsset,
};
use crate::core::runtime::tasks::{TaskPool, TaskPoolKind, TaskPools};

const DEFAULT_ASSET_WORKER_QUEUE_DEPTH_PER_THREAD: usize = 2;

impl Default for ProjectAssetManager {
    fn default() -> Self {
        Self::new(TaskPools::default().io().clone())
    }
}

impl ProjectAssetManager {
    pub fn new(worker_task_pool: TaskPool) -> Self {
        assert_eq!(
            worker_task_pool.kind(),
            TaskPoolKind::Io,
            "ProjectAssetManager requires the runtime IO task pool"
        );
        Self {
            worker_task_pool,
            project_generation_gate: Arc::new(RwLock::new(())),
            project_preparation_epoch: Arc::new(AtomicU64::new(0)),
            project: Arc::new(RwLock::new(None)),
            project_source_paths: Arc::new(RwLock::new(ProjectSourcePathIndex::new())),
            asset_importers: Arc::new(RwLock::new(AssetImporterRegistry::default())),
            resource_manager: resource_manager_with_builtins(),
            residency_stripes: Arc::new(std::array::from_fn(|_| Mutex::new(()))),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            generation_wake_subscribers: Arc::new(Mutex::new(Vec::new())),
            watch_error_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher_activation: Arc::new(Mutex::new(Option::<Arc<ProjectWatcherActivation>>::None)),
            watch_refresh_gate: Arc::new(Mutex::new(())),
            watch_diagnostics: Arc::new(Mutex::new(Default::default())),
            watchers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn_worker_pool_with_frame_sampler(
        &self,
    ) -> (AssetWorkerPool, AssetWorkerPoolFrameSampler) {
        let queue_depth = self
            .worker_task_pool
            .parallelism()
            .saturating_mul(DEFAULT_ASSET_WORKER_QUEUE_DEPTH_PER_THREAD);
        let pool = AssetWorkerPool::new(
            self.worker_task_pool.clone(),
            AssetWorkerPoolOptions::new().with_queue_depth(queue_depth),
        );
        let sampler = AssetWorkerPoolFrameSampler::from_pool(&pool);
        (pool, sampler)
    }

    pub fn worker_task_pool(&self) -> &TaskPool {
        &self.worker_task_pool
    }

    pub fn default_worker_count(&self) -> usize {
        self.worker_task_pool.parallelism()
    }

    pub fn default_worker_budget_source(&self) -> AssetWorkerThreadBudgetSource {
        AssetWorkerThreadBudgetSource::TaskPoolIo
    }

    pub fn resource_manager(&self) -> ResourceManager {
        self.resource_manager.clone()
    }

    pub fn register_asset_importer(
        &self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), CoreError> {
        self.register_asset_importer_arc(Arc::new(importer))
    }

    pub fn register_asset_importer_arc(
        &self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), CoreError> {
        {
            let project = self.project_read();
            if let Some(project) = project.as_ref() {
                let mut active_registry = project.importer().registry().clone();
                active_registry
                    .register_arc(importer.clone())
                    .map_err(|error| asset_error_message(error.to_string()))?;
            }
        }

        self.importer_registry_write()
            .register_arc(importer.clone())
            .map_err(|error| asset_error_message(error.to_string()))?;
        self.begin_project_preparation();

        let mut project = self.project_write();
        if let Some(project) = project.as_mut() {
            project
                .register_asset_importer_arc(importer)
                .map_err(|error| asset_error_message(error.to_string()))?;
        }
        Ok(())
    }

    pub fn asset_importer_capability_reports(&self) -> Vec<AssetImporterCapabilityReport> {
        let project = self.project_read();
        if let Some(project) = project.as_ref() {
            return project.importer().capability_reports();
        }
        self.active_importer_registry().capability_reports()
    }

    pub fn asset_importer_capability_report_for_source(
        &self,
        source_path: &std::path::Path,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError> {
        let project = self.project_read();
        if let Some(project) = project.as_ref() {
            return project.importer().capability_report_for_source(source_path);
        }
        self.active_importer_registry()
            .capability_report_for_source(source_path)
    }

    #[cfg(test)]
    pub(crate) fn register_first_wave_plugin_fixture_importers_for_test(
        &self,
    ) -> Result<(), CoreError> {
        for importer in AssetImporter::first_wave_plugin_fixture_importers_for_test() {
            self.register_asset_importer(importer)?;
        }
        Ok(())
    }

    pub fn resolve_asset_id(&self, uri: &AssetUri) -> Option<AssetId> {
        self.resource_manager()
            .registry()
            .get_by_locator(uri)
            .map(|record| record.id())
    }

    pub fn current_project_manager(&self) -> Option<ProjectManager> {
        self.project_read().clone()
    }

    pub fn ibl_bake_artifact_cache_store(&self) -> Option<IblBakeArtifactCacheStore> {
        let project = self.project_read();
        project
            .as_ref()
            .map(|project| IblBakeArtifactCacheStore::new(project.paths().cache_root()))
    }

    pub fn runtime_ref_count(&self, id: AssetId) -> Option<usize> {
        self.resource_manager().ref_count(id)
    }

    pub fn runtime_resource_state(&self, id: AssetId) -> Option<RuntimeResourceState> {
        self.resource_manager().runtime_state(id)
    }

    pub fn load_shader_asset_by_uri(&self, uri: &AssetUri) -> Result<ShaderAsset, CoreError> {
        let id = self
            .resolve_asset_id(uri)
            .ok_or_else(|| asset_error_message(format!("missing shader locator {uri}")))?;
        self.load_shader_asset(id)
    }

    fn active_importer_registry(&self) -> AssetImporterRegistry {
        let mut registry = AssetImporter::default().registry().clone();
        for importer in self.importer_registry_read().importers() {
            let _ = registry.register_arc(importer);
        }
        registry
    }
}
