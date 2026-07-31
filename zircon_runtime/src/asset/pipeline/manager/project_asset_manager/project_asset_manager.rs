use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::framework::channel::ChannelSender;
use crate::core::resource::{ResourceManager, ResourceScheme};
use crate::core::runtime::tasks::TaskPool;

use crate::asset::AssetImporterRegistry;
use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchError, AssetWatcher};

pub(in crate::asset::pipeline::manager) type ProjectSourcePathIndex =
    HashMap<ResourceScheme, HashMap<String, PathBuf>>;

pub(in crate::asset::pipeline::manager) struct ProjectWatcherActivation {
    pub(in crate::asset::pipeline::manager) state: Mutex<ProjectWatcherActivationState>,
}

pub(in crate::asset::pipeline::manager) struct ProjectWatcherActivationState {
    pub(in crate::asset::pipeline::manager) lifecycle: ProjectWatcherLifecycle,
    pub(in crate::asset::pipeline::manager) changes: Vec<AssetChange>,
    pub(in crate::asset::pipeline::manager) errors: Vec<AssetWatchError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::asset::pipeline::manager) enum ProjectWatcherLifecycle {
    Pending,
    Draining,
    Active,
    Retired,
}

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) worker_task_pool: TaskPool,
    pub(in crate::asset::pipeline::manager) project_generation_gate: Arc<RwLock<()>>,
    pub(in crate::asset::pipeline::manager) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(in crate::asset::pipeline::manager) project_source_paths:
        Arc<RwLock<ProjectSourcePathIndex>>,
    pub(in crate::asset::pipeline::manager) asset_importers: Arc<RwLock<AssetImporterRegistry>>,
    pub(in crate::asset::pipeline::manager) resource_manager: ResourceManager,
    pub(in crate::asset::pipeline::manager) change_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetChange>>>>,
    pub(in crate::asset::pipeline::manager) watch_error_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetWatchError>>>>,
    pub(in crate::asset::pipeline::manager) watcher_activation:
        Arc<Mutex<Option<Arc<ProjectWatcherActivation>>>>,
    pub(in crate::asset::pipeline::manager) watchers: Arc<Mutex<Vec<AssetWatcher>>>,
}
