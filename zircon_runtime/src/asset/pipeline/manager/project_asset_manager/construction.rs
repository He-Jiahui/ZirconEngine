use std::sync::{Arc, Mutex, RwLock};

use crate::core::resource::{ResourceManager, RuntimeResourceState};
use crate::core::CoreError;

use super::super::builtins::resource_manager_with_builtins;
use super::super::errors::asset_error_message;
use super::ProjectAssetManager;
use crate::asset::project::ProjectManager;
use crate::asset::worker_pool::AssetWorkerPool;
#[cfg(test)]
use crate::asset::AssetImporter;
use crate::asset::{AssetId, AssetImporterHandler, AssetImporterRegistry, AssetUri, ShaderAsset};

impl Default for ProjectAssetManager {
    fn default() -> Self {
        Self {
            default_worker_count: default_worker_count_for_system(),
            project: Arc::new(RwLock::new(None)),
            asset_importers: Arc::new(RwLock::new(AssetImporterRegistry::default())),
            resource_manager: resource_manager_with_builtins(),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }
}

impl ProjectAssetManager {
    pub fn new(default_worker_count: usize) -> Self {
        Self {
            default_worker_count: default_worker_count.max(1),
            project: Arc::new(RwLock::new(None)),
            asset_importers: Arc::new(RwLock::new(AssetImporterRegistry::default())),
            resource_manager: resource_manager_with_builtins(),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn spawn_worker_pool(&self) -> Result<AssetWorkerPool, crate::core::ZirconError> {
        AssetWorkerPool::new(self.default_worker_count)
    }

    pub fn default_worker_count(&self) -> usize {
        self.default_worker_count
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

        self.asset_importers
            .write()
            .expect("asset importer registry lock poisoned")
            .register_arc(importer.clone())
            .map_err(|error| asset_error_message(error.to_string()))?;

        let mut project = self.project_write();
        if let Some(project) = project.as_mut() {
            project
                .register_asset_importer_arc(importer)
                .map_err(|error| asset_error_message(error.to_string()))?;
        }
        Ok(())
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
}

fn default_worker_count_for_system() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1)
}
