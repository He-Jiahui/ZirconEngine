use std::sync::{Arc, Mutex, RwLock};

use zircon_core::ChannelSender;
use zircon_resource::ResourceManager;

use crate::project::ProjectManager;
use crate::watch::{AssetChange, AssetWatcher};
use crate::DefaultEditorAssetManager as EditorAssetManagerService;

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(in crate::pipeline::manager) default_worker_count: usize,
    pub(in crate::pipeline::manager) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(in crate::pipeline::manager) resource_manager: ResourceManager,
    pub(in crate::pipeline::manager) editor_asset_manager: Arc<EditorAssetManagerService>,
    pub(in crate::pipeline::manager) change_subscribers:
        Arc<Mutex<Vec<ChannelSender<AssetChange>>>>,
    pub(in crate::pipeline::manager) watcher: Arc<Mutex<Option<AssetWatcher>>>,
}
