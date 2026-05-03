use std::sync::{Arc, Mutex, RwLock};

use crate::core::resource::ResourceManager;
use crate::core::ChannelSender;

use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatcher};
use crate::asset::AssetImporterRegistry;

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) default_worker_count: usize,
    pub(in crate::asset::pipeline::manager) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(in crate::asset::pipeline::manager) asset_importers: Arc<RwLock<AssetImporterRegistry>>,
    pub(in crate::asset::pipeline::manager) resource_manager: ResourceManager,
    pub(in crate::asset::pipeline::manager) change_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetChange>>>>,
    pub(in crate::asset::pipeline::manager) watcher: Arc<Mutex<Option<AssetWatcher>>>,
}
