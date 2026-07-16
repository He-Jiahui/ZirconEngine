use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::channel::ChannelSender;
use zircon_runtime::core::manager::{ManagerResolver, ManagerServiceHandle};
use zircon_runtime::core::{CoreError, CoreHandle};

use super::super::EditorAssetChangeRecord;

mod asset_details;
mod broadcast;
mod catalog_snapshot;
mod editor_asset_error;
mod editor_asset_state;
mod parse_uuid;
mod preview_trait_bridge;
mod record_access;
mod record_to_view;
mod reference_to_view;
mod subscribe_editor_asset_changes;

pub(super) use editor_asset_state::EditorAssetState;

#[derive(Clone, Debug)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    project_asset_manager: Option<ProjectAssetManagerAccess>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<EditorAssetChangeRecord>>>>,
}

#[derive(Clone, Debug)]
struct ProjectAssetManagerAccess {
    resolver: ManagerResolver,
    handle: ManagerServiceHandle<ProjectAssetManager>,
}

impl Default for DefaultEditorAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultEditorAssetManager {
    pub(super) fn lock_change_subscribers(
        &self,
    ) -> MutexGuard<'_, Vec<ChannelSender<EditorAssetChangeRecord>>> {
        self.change_subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            project_asset_manager: None,
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_runtime_project_manager(
        core: CoreHandle,
        handle: ManagerServiceHandle<ProjectAssetManager>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            project_asset_manager: Some(ProjectAssetManagerAccess {
                resolver: ManagerResolver::new(core),
                handle,
            }),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        let Some(access) = self.project_asset_manager.as_ref() else {
            return Ok(());
        };
        let project_asset_manager = access.resolver.resolve(access.handle.clone())?;
        let Some(project) = project_asset_manager.current_project_manager() else {
            return Ok(());
        };
        self.sync_from_project(project)
            .map_err(editor_asset_error::editor_asset_error)
    }
}
