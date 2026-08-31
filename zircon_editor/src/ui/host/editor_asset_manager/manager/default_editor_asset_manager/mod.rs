use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use zircon_runtime::asset::AssetManager;
use zircon_runtime::core::CoreError;

use super::super::EditorAssetChangeHub;
use crate::core::asset::{EditorAssetImportFlow, EditorAssetIndex, EditorModelImportTicket};
use crate::core::jobs::EditorJobSystem;
use crate::core::logging::EditorLogService;
use crate::ui::host::runtime_services::EditorProjectAssetRuntimeAccess;

mod asset_details;
mod asset_refactor;
mod broadcast;
mod catalog_snapshot;
mod editor_asset_error;
mod editor_asset_state;
mod parse_uuid;
mod preview_trait_bridge;
mod project_deactivation;
mod project_deletion;
mod project_relocation;
mod subscribe_editor_asset_changes;
mod watch_projection;

pub(super) use editor_asset_state::{
    lock_editor_asset_gate_recovering_poison, read_editor_asset_state_recovering_poison,
    write_editor_asset_state_recovering_poison, EditorAssetState,
};

#[derive(Clone)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    pub(super) publish_gate: Arc<Mutex<()>>,
    project_asset_manager: Option<EditorProjectAssetRuntimeAccess>,
    pub(super) change_stream: EditorAssetChangeHub,
    pub(super) preview_jobs: Option<EditorJobSystem>,
    import_flow: Arc<Mutex<Option<EditorAssetImportFlow>>>,
    import_logs: Option<Arc<EditorLogService>>,
    pub(super) source_sync_epoch: Arc<AtomicU64>,
    pub(super) source_sync_gate: Arc<Mutex<()>>,
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
            import_flow: Arc::new(Mutex::new(None)),
            import_logs: None,
            source_sync_epoch: Arc::new(AtomicU64::new(0)),
            source_sync_gate: Arc::new(Mutex::new(())),
        }
    }

    pub(in crate::ui::host) fn with_runtime_project_manager(
        project_asset_manager: EditorProjectAssetRuntimeAccess,
        preview_jobs: EditorJobSystem,
        import_logs: Arc<EditorLogService>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorAssetState::default())),
            publish_gate: Arc::new(Mutex::new(())),
            project_asset_manager: Some(project_asset_manager),
            change_stream: EditorAssetChangeHub::default(),
            preview_jobs: Some(preview_jobs),
            import_flow: Arc::new(Mutex::new(None)),
            import_logs: Some(import_logs),
            source_sync_epoch: Arc::new(AtomicU64::new(0)),
            source_sync_gate: Arc::new(Mutex::new(())),
        }
    }

    pub fn refresh_from_runtime_project(&self) -> Result<(), CoreError> {
        let Some(access) = self.project_asset_manager.as_ref() else {
            return Ok(());
        };
        let project_asset_manager = access.project_asset_manager()?;
        let Some(snapshot) = project_asset_manager.current_project_generation_snapshot() else {
            self.deactivate_runtime_project();
            return Ok(());
        };
        let (project, generation) = snapshot.into_parts();
        self.sync_from_runtime_project_generation(
            project_asset_manager.as_ref(),
            project,
            &generation,
        )
        .map_err(editor_asset_error::editor_asset_error)
    }

    pub fn submit_model_import(
        &self,
        source_path: std::path::PathBuf,
    ) -> Result<EditorModelImportTicket, CoreError> {
        self.current_import_flow()?.submit_model_source(source_path)
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn clear_import_flow(&self) {
        self.import_flow
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    fn current_import_flow(&self) -> Result<EditorAssetImportFlow, CoreError> {
        let mut flow = self
            .import_flow
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(flow) = flow.as_ref() {
            return Ok(flow.clone());
        }
        let asset_index = self
            .read_state_recovering_poison()
            .asset_index
            .clone()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorAssetIndex".to_owned()))?;
        let access = self
            .project_asset_manager
            .as_ref()
            .ok_or_else(|| CoreError::ServiceUnavailable("ProjectAssetManager".to_owned()))?;
        let jobs = self
            .preview_jobs
            .as_ref()
            .cloned()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorJobSystem".to_owned()))?;
        let logs = self
            .import_logs
            .as_ref()
            .cloned()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorLogService".to_owned()))?;
        let project_manager = access.project_asset_manager()?;
        let asset_manager: Arc<dyn AssetManager> = project_manager;
        let created = EditorAssetImportFlow::new(jobs, asset_manager, asset_index, logs);
        *flow = Some(created.clone());
        Ok(created)
    }
}
