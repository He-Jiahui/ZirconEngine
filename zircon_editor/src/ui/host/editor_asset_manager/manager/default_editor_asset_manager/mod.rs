use std::sync::{Arc, Mutex, RwLock};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::ChannelSender;

use super::super::EditorAssetChangeRecord;

mod asset_details;
mod broadcast;
mod catalog_snapshot;
mod editor_asset_error;
mod editor_asset_state;
mod parse_uuid;
mod preview_trait_bridge;
mod record_access;
mod record_to_facade;
mod reference_to_facade;
mod subscribe_editor_asset_changes;

pub(super) use editor_asset_state::EditorAssetState;

#[derive(Clone, Debug)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    project_asset_manager: Option<Arc<ProjectAssetManager>>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<EditorAssetChangeRecord>>>>,
}

impl Default for DefaultEditorAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultEditorAssetManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            project_asset_manager: None,
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_project_asset_manager(project_asset_manager: Arc<ProjectAssetManager>) -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            project_asset_manager: Some(project_asset_manager),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn refresh_from_runtime_project(
        &self,
    ) -> Result<(), zircon_runtime::asset::importer::AssetImportError> {
        let Some(project_asset_manager) = self.project_asset_manager.as_ref() else {
            return Ok(());
        };
        let Some(project) = project_asset_manager.current_project_manager() else {
            return Ok(());
        };
        self.sync_from_project(project)
    }
}
