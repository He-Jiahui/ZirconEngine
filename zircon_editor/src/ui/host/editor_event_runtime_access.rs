use zircon_runtime::scene::LevelSystem;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceRecord,
};

use crate::core::editor_event::EditorEventRuntime;
use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEnvelope, EditorEventJournal, EditorEventRecord,
    EditorEventSource,
};
use crate::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorUiTemplateDescriptor,
};
use crate::core::editor_operation::EditorOperationStack;
use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot};
use crate::ui::activity::ActivityViewDescriptor;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
};
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, EditorDataSnapshot};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentDataSourceDescriptor,
    UiComponentEventEnvelope,
};

impl EditorEventRuntime {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        self.lock_inner().state.snapshot()
    }

    pub fn current_layout(&self) -> WorkbenchLayout {
        self.lock_inner().manager.current_layout()
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.lock_inner().manager.descriptors()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.lock_inner().manager.current_view_instances()
    }

    pub fn chrome_snapshot(&self) -> EditorChromeSnapshot {
        let inner = self.lock_inner();
        let descriptors = inner.manager.descriptors();
        Self::build_chrome_locked(&inner, descriptors)
    }

    pub fn preset_names(&self) -> Vec<String> {
        self.lock_inner().manager.preset_names().unwrap_or_default()
    }

    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        self.lock_inner().state.render_snapshot()
    }

    pub fn render_frame_extract(&self) -> Option<RenderFrameExtract> {
        self.lock_inner().state.render_frame_extract()
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        self.lock_inner().state.render_frame_submission()
    }

    pub fn viewport_state(&self) -> crate::scene::viewport::ViewportState {
        self.lock_inner().state.viewport_state()
    }

    pub fn set_status_line(&self, message: impl Into<String>) {
        let mut inner = self.lock_inner();
        inner.state.set_status_line(message);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub(crate) fn dispatch_ui_component_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        let mut inner = self.lock_inner();
        let manager = inner.manager.clone();
        let result =
            crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::apply_envelope(
                &mut inner.state,
                manager.as_ref(),
                envelope,
            )?;
        if result.refresh_projection {
            Self::refresh_reflection_locked(&mut inner);
        }
        Ok(result)
    }

    pub(crate) fn ui_component_data_sources(&self) -> Vec<UiComponentDataSourceDescriptor> {
        crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::data_sources()
    }

    pub fn set_session_mode(&self, session_mode: EditorSessionMode) {
        let mut inner = self.lock_inner();
        inner.state.set_session_mode(session_mode);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn set_welcome_snapshot(&self, welcome: WelcomePaneSnapshot) {
        let mut inner = self.lock_inner();
        inner.state.set_welcome_snapshot(welcome);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_catalog(&self, catalog: EditorAssetCatalogSnapshotRecord) {
        let mut inner = self.lock_inner();
        inner.state.sync_asset_catalog(catalog);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_resources(&self, resources: Vec<ResourceRecord>) {
        let mut inner = self.lock_inner();
        inner.state.sync_asset_resources(resources);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn sync_asset_details(&self, details: Option<EditorAssetDetailsRecord>) {
        let mut inner = self.lock_inner();
        inner.state.sync_asset_details(details);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn replace_world(&self, world: LevelSystem, project_path: impl Into<String>) {
        let mut inner = self.lock_inner();
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
        let mut inner = self.lock_inner();
        let changed = inner
            .state
            .import_mesh_asset(model, material, display_path)?;
        Self::refresh_reflection_locked(&mut inner);
        Ok(changed)
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.lock_inner().journal.clone()
    }

    pub fn operation_stack(&self) -> EditorOperationStack {
        self.lock_inner().operation_stack.clone()
    }

    pub fn activity_view_descriptor(&self, view_id: &str) -> Option<ActivityViewDescriptor> {
        self.lock_inner()
            .control_service
            .activity_view(view_id)
            .cloned()
    }

    pub fn component_drawer_descriptor(
        &self,
        component_type: &str,
    ) -> Option<ComponentDrawerDescriptor> {
        let inner = self.lock_inner();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        inner
            .editor_extensions
            .iter()
            .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
            .flat_map(|registration| registration.registry().component_drawers())
            .find(|descriptor| descriptor.component_type() == component_type)
            .cloned()
    }

    pub fn ui_template_descriptor(&self, id: &str) -> Option<EditorUiTemplateDescriptor> {
        let inner = self.lock_inner();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        inner
            .editor_extensions
            .iter()
            .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
            .flat_map(|registration| registration.registry().ui_templates())
            .find(|descriptor| descriptor.id() == id)
            .cloned()
    }

    pub fn asset_importers_for_extension(&self, extension: &str) -> Vec<AssetImporterDescriptor> {
        let normalized = extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let inner = self.lock_inner();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        inner
            .editor_extensions
            .iter()
            .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
            .flat_map(|registration| registration.registry().asset_importers())
            .filter(|descriptor| {
                descriptor
                    .source_extensions()
                    .iter()
                    .any(|candidate| candidate == &normalized)
            })
            .cloned()
            .collect()
    }

    pub fn asset_editor_descriptor(&self, asset_kind: &str) -> Option<AssetEditorDescriptor> {
        let inner = self.lock_inner();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        inner
            .editor_extensions
            .iter()
            .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
            .flat_map(|registration| registration.registry().asset_editors())
            .find(|descriptor| descriptor.asset_kind() == asset_kind)
            .cloned()
    }

    pub fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_envelope(self, envelope)
    }

    pub fn dispatch_binding(
        &self,
        binding: crate::ui::binding::EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_binding(self, binding.as_ui_binding(), source)
    }

    pub fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_event(self, source, event)
    }
}
