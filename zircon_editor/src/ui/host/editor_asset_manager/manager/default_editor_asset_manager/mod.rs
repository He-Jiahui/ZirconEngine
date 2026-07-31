use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::manager::{ManagerResolver, ManagerServiceHandle};
use zircon_runtime::core::{CoreError, CoreHandle};

use super::super::EditorAssetChangeHub;
use crate::core::jobs::EditorJobSystem;

mod asset_details;
mod broadcast;
mod catalog_snapshot;
mod editor_asset_error;
mod editor_asset_state;
mod parse_uuid;
mod preview_trait_bridge;
mod project_deactivation;
mod record_access;
mod subscribe_editor_asset_changes;

pub(super) use editor_asset_state::EditorAssetState;

#[derive(Clone)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    pub(super) publish_gate: Arc<Mutex<()>>,
    project_asset_manager: Option<ProjectAssetManagerAccess>,
    pub(super) change_stream: EditorAssetChangeHub,
    pub(super) preview_jobs: Option<EditorJobSystem>,
    pub(super) source_sync_epoch: Arc<AtomicU64>,
    pub(super) source_sync_gate: Arc<Mutex<()>>,
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
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            publish_gate: Arc::new(Mutex::new(())),
            project_asset_manager: None,
            change_stream: EditorAssetChangeHub::default(),
            preview_jobs: None,
            source_sync_epoch: Arc::new(AtomicU64::new(0)),
            source_sync_gate: Arc::new(Mutex::new(())),
        }
    }

    pub fn with_runtime_project_manager(
        core: CoreHandle,
        handle: ManagerServiceHandle<ProjectAssetManager>,
        preview_jobs: EditorJobSystem,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            publish_gate: Arc::new(Mutex::new(())),
            project_asset_manager: Some(ProjectAssetManagerAccess {
                resolver: ManagerResolver::new(core),
                handle,
            }),
            change_stream: EditorAssetChangeHub::default(),
            preview_jobs: Some(preview_jobs),
            source_sync_epoch: Arc::new(AtomicU64::new(0)),
            source_sync_gate: Arc::new(Mutex::new(())),
        }
    }

    pub fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        let Some(access) = self.project_asset_manager.as_ref() else {
            return Ok(());
        };
        let project_asset_manager = access.resolver.resolve(access.handle.clone())?;
        let Some(project) = project_asset_manager.current_project_manager() else {
            self.deactivate_runtime_project()?;
            return Ok(());
        };
        self.sync_from_project(project)
            .map_err(editor_asset_error::editor_asset_error)
    }
}
