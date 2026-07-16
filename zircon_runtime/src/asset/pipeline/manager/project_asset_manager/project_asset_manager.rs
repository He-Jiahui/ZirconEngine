use std::sync::{Arc, Mutex, RwLock};

use crate::core::framework::channel::ChannelSender;
use crate::core::resource::ResourceManager;
use crate::core::runtime::tasks::TaskPool;

use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchError, AssetWatcher};
use crate::asset::AssetImporterRegistry;

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) worker_task_pool: TaskPool,
    pub(in crate::asset::pipeline::manager) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(in crate::asset::pipeline::manager) asset_importers: Arc<RwLock<AssetImporterRegistry>>,
    pub(in crate::asset::pipeline::manager) resource_manager: ResourceManager,
    pub(in crate::asset::pipeline::manager) change_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetChange>>>>,
    pub(in crate::asset::pipeline::manager) watch_error_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetWatchError>>>>,
    pub(in crate::asset::pipeline::manager) watchers: Arc<Mutex<Vec<AssetWatcher>>>,
}
