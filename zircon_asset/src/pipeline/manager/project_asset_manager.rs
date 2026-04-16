use std::sync::{Arc, Mutex, RwLock};

use zircon_core::ChannelSender;
use zircon_manager::AssetChangeRecord;

use crate::{
    AssetWatcher, DefaultEditorAssetManager as EditorAssetManagerService, ProjectManager,
    ResourceManager,
};

#[derive(Clone)]
pub struct ProjectAssetManager {
    pub(super) default_worker_count: usize,
    pub(super) project: Arc<RwLock<Option<ProjectManager>>>,
    pub(super) resource_manager: ResourceManager,
    pub(super) editor_asset_manager: Arc<EditorAssetManagerService>,
    pub(super) change_subscribers: Arc<Mutex<Vec<ChannelSender<AssetChangeRecord>>>>,
    pub(super) watcher: Arc<Mutex<Option<AssetWatcher>>>,
}
