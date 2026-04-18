use std::sync::{Arc, Mutex, RwLock};

use zircon_core::CoreError;
use zircon_resource::{ResourceManager, RuntimeResourceState};

use super::super::builtins::resource_manager_with_builtins;
use super::super::errors::asset_error_message;
use super::ProjectAssetManager;
use crate::worker_pool::AssetWorkerPool;
use crate::{
    AssetId, AssetUri, DefaultEditorAssetManager as EditorAssetManagerService, ShaderAsset,
};

impl Default for ProjectAssetManager {
    fn default() -> Self {
        Self {
            default_worker_count: default_worker_count_for_system(),
            project: Arc::new(RwLock::new(None)),
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager: Arc::new(EditorAssetManagerService::default()),
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
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager: Arc::new(EditorAssetManagerService::default()),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_editor_asset_manager(
        default_worker_count: usize,
        editor_asset_manager: Arc<EditorAssetManagerService>,
    ) -> Self {
        Self {
            default_worker_count: default_worker_count.max(1),
            project: Arc::new(RwLock::new(None)),
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager,
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn spawn_worker_pool(&self) -> Result<AssetWorkerPool, zircon_core::ZirconError> {
        AssetWorkerPool::new(self.default_worker_count)
    }

    pub fn default_worker_count(&self) -> usize {
        self.default_worker_count
    }

    pub fn resource_manager(&self) -> ResourceManager {
        self.resource_manager.clone()
    }

    pub fn editor_asset_manager(&self) -> Arc<EditorAssetManagerService> {
        self.editor_asset_manager.clone()
    }

    pub fn resolve_asset_id(&self, uri: &AssetUri) -> Option<AssetId> {
        self.resource_manager()
            .registry()
            .get_by_locator(uri)
            .map(|record| record.id())
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
