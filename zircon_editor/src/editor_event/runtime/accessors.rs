use zircon_asset::{EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceRecord};
use zircon_scene::LevelSystem;

use crate::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventJournal, EditorEventRecord, EditorEventSource,
};
use crate::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::{EditorChromeSnapshot, EditorDataSnapshot, ViewDescriptor};

use super::editor_event_dispatcher::EditorEventDispatcher;
use super::editor_event_runtime::EditorEventRuntime;

impl EditorEventRuntime {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        self.inner.lock().unwrap().state.snapshot()
    }

    pub fn current_layout(&self) -> crate::WorkbenchLayout {
        self.inner.lock().unwrap().manager.current_layout()
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.inner.lock().unwrap().manager.descriptors()
    }

    pub fn current_view_instances(&self) -> Vec<crate::ViewInstance> {
        self.inner.lock().unwrap().manager.current_view_instances()
    }

    pub fn chrome_snapshot(&self) -> EditorChromeSnapshot {
        let inner = self.inner.lock().unwrap();
        let descriptors = inner.manager.descriptors();
        Self::build_chrome_locked(&inner, descriptors)
    }

    pub fn preset_names(&self) -> Vec<String> {
        self.inner
            .lock()
            .unwrap()
            .manager
            .preset_names()
            .unwrap_or_default()
    }

    pub fn render_snapshot(&self) -> Option<zircon_scene::RenderSceneSnapshot> {
        self.inner.lock().unwrap().state.render_snapshot()
    }

    pub fn render_frame_extract(&self) -> Option<zircon_scene::RenderFrameExtract> {
        self.inner.lock().unwrap().state.render_frame_extract()
    }

    pub fn viewport_state(&self) -> crate::ViewportState {
        self.inner.lock().unwrap().state.viewport_state()
    }

    pub fn set_status_line(&self, message: impl Into<String>) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.set_status_line(message);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn set_session_mode(&self, session_mode: EditorSessionMode) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.set_session_mode(session_mode);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn set_welcome_snapshot(&self, welcome: WelcomePaneSnapshot) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.set_welcome_snapshot(welcome);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_catalog(&self, catalog: EditorAssetCatalogSnapshotRecord) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.sync_asset_catalog(catalog);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_resources(&self, resources: Vec<ResourceRecord>) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.sync_asset_resources(resources);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_details(&self, details: Option<EditorAssetDetailsRecord>) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.sync_asset_details(details);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn replace_world(&self, world: LevelSystem, project_path: impl Into<String>) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.replace_world(world, project_path);
        inner.dragging_gizmo = false;
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn import_mesh_asset(
        &self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, String> {
        let mut inner = self.inner.lock().unwrap();
        let changed = inner
            .state
            .import_mesh_asset(model, material, display_path)?;
        Self::refresh_reflection_locked(&mut inner);
        Ok(changed)
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.inner.lock().unwrap().journal.clone()
    }

    pub fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_envelope(self, envelope)
    }

    pub fn dispatch_binding(
        &self,
        binding: zircon_editor_ui::EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_binding(self, binding, source)
    }

    pub fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_event(self, source, event)
    }
}
