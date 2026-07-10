use zircon_runtime::scene::LevelSystem;
use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceRecord,
};

use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEnvelope, EditorEventJournal, EditorEventRecord,
    EditorEventSource,
};
use crate::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorUiTemplateDescriptor,
};
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::editor_operation::EditorOperationStack;
use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot};
use crate::ui::activity::ActivityViewDescriptor;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, EditorDataSnapshot, StatusTaskProgressSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEventEnvelope,
};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiInputDispatchResult, UiInputEvent, UiKeyboardInputEvent,
};

impl EditorHostEventController {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        let inner = self.shell().lock();
        let component_drawers = Self::active_component_drawers_for_shell(&inner);
        inner
            .state
            .snapshot_with_component_drawers(&component_drawers)
    }

    pub fn current_layout(&self) -> WorkbenchLayout {
        self.shell().lock().manager.current_layout()
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.shell().lock().manager.descriptors()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.shell().lock().manager.current_view_instances()
    }

    pub fn chrome_snapshot(&self) -> EditorChromeSnapshot {
        let inner = self.shell().lock();
        let descriptors = inner.manager.descriptors();
        Self::build_chrome_for_shell(&inner, descriptors)
    }

    pub fn preset_names(&self) -> Vec<String> {
        self.shell()
            .lock()
            .manager
            .preset_names()
            .unwrap_or_default()
    }

    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        self.shell().lock().state.render_snapshot()
    }

    pub fn render_frame_extract(&self) -> Option<RenderFrameExtract> {
        self.shell().lock().state.render_frame_extract()
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        self.shell().lock().state.render_frame_submission()
    }

    pub fn viewport_state(&self) -> crate::scene::viewport::ViewportState {
        self.shell().lock().state.viewport_state()
    }

    pub fn set_status_line(&self, message: impl Into<String>) {
        self.shell().lock().state.set_status_line(message);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn status_line(&self) -> String {
        self.shell().lock().state.status_line.clone()
    }

    pub fn set_status_task_progress(&self, progress: Option<StatusTaskProgressSnapshot>) {
        self.shell().lock().state.set_status_task_progress(progress);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn status_task_progress(&self) -> Option<StatusTaskProgressSnapshot> {
        self.shell().lock().state.status_task_progress.clone()
    }

    pub(crate) fn dispatch_ui_component_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        if envelope.target.domain == "component_drawer" {
            return self.dispatch_component_drawer_adapter_event(envelope);
        }
        if envelope.target.domain
            == crate::ui::template_runtime::component_adapter::command::COMMAND_DOMAIN
        {
            return self.dispatch_command_component_adapter_event(envelope);
        }
        let result = {
            let mut inner = self.shell().lock();
            let manager = inner.manager.clone();
            crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::apply_envelope(
                    &mut inner.state,
                    manager.as_ref(),
                    envelope,
                )?
        };
        if result.refresh_projection {
            self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
        }
        Ok(result)
    }

    pub(crate) fn dispatch_unhandled_input_keymap_command(
        &self,
        result: &UiInputDispatchResult,
        source: EditorEventSource,
    ) -> Result<Option<EditorEventRecord>, String> {
        if result.reply.disposition != UiDispatchDisposition::Unhandled {
            return Ok(None);
        }
        let UiInputEvent::Keyboard(keyboard) = &result.event else {
            return Ok(None);
        };
        self.dispatch_keyboard_keymap_command(keyboard, source)
    }

    pub(crate) fn dispatch_keyboard_keymap_command(
        &self,
        keyboard: &UiKeyboardInputEvent,
        source: EditorEventSource,
    ) -> Result<Option<EditorEventRecord>, String> {
        let keymap = crate::ui::host::EditorKeymap::default_workbench();
        let Some(command_id) = keymap.resolve_keyboard_input(keyboard) else {
            return Ok(None);
        };
        self.dispatch_keymap_command_id(command_id, source)
            .map(Some)
    }

    fn dispatch_keymap_command_id(
        &self,
        command_id: &str,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let binding = crate::ui::binding::EditorUiBinding::new(
            "EditorKeymap",
            command_id,
            crate::ui::binding::EditorUiEventKind::Submit,
            crate::ui::binding::EditorUiBindingPayload::editor_command(command_id),
        );
        self.dispatch_binding(binding, source)
    }

    fn dispatch_component_drawer_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        let component_type = envelope.target.subject.as_deref().ok_or_else(|| {
            UiComponentAdapterError::MissingSource {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                source_name: "subject".to_string(),
            }
        })?;
        let drawer = self.component_drawer_descriptor(component_type);
        let operation_path =
            crate::ui::template_runtime::component_adapter::component_drawer::validate_component_drawer_envelope(
                envelope,
                drawer.as_ref(),
            )?;
        let (source, invocation) = crate::ui::template_runtime::component_adapter::component_drawer::component_drawer_operation_invocation(operation_path.clone());
        self.invoke_operation(source, invocation).map_err(|error| {
            UiComponentAdapterError::HostMutation {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                reason: error,
            }
        })?;
        Ok(
            crate::ui::template_runtime::component_adapter::component_drawer::component_drawer_operation_result(
                &operation_path,
            ),
        )
    }

    fn dispatch_command_component_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        let binding =
            crate::ui::template_runtime::component_adapter::command::editor_command_binding_for_envelope(
                envelope,
            )?;
        let command_id = match binding.payload() {
            crate::ui::binding::EditorUiBindingPayload::EditorCommand { command_id } => {
                command_id.clone()
            }
            _ => unreachable!("command adapter must build EditorCommand bindings"),
        };

        self.dispatch_binding(binding, EditorEventSource::RetainedHost)
            .map_err(|error| UiComponentAdapterError::HostMutation {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                reason: error,
            })?;
        Ok(
            crate::ui::template_runtime::component_adapter::command::command_adapter_result(
                &command_id,
            ),
        )
    }

    #[cfg(test)]
    pub(crate) fn ui_component_data_sources(
        &self,
    ) -> Vec<zircon_runtime_interface::ui::component::UiComponentDataSourceDescriptor> {
        crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::data_sources()
    }

    pub fn set_session_mode(&self, session_mode: EditorSessionMode) {
        self.shell().lock().state.set_session_mode(session_mode);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn set_welcome_snapshot(&self, welcome: WelcomePaneSnapshot) {
        self.shell().lock().state.set_welcome_snapshot(welcome);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_catalog(&self, catalog: EditorAssetCatalogSnapshotRecord) {
        self.shell().lock().state.sync_asset_catalog(catalog);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_resources(&self, resources: Vec<ResourceRecord>) {
        self.shell().lock().state.sync_asset_resources(resources);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_details(&self, details: Option<EditorAssetDetailsRecord>) {
        self.shell().lock().state.sync_asset_details(details);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn replace_world(&self, world: LevelSystem, project_path: impl Into<String>) {
        self.shell().lock().state.replace_world(world, project_path);
        self.gizmo_drag().clear();
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
    }

    pub fn import_mesh_asset(
        &self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, String> {
        let changed = self
            .shell()
            .lock()
            .state
            .import_mesh_asset(model, material, display_path)?;
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(changed)
    }

    pub fn journal(&self) -> EditorEventJournal {
        self.context().events().journal()
    }

    pub fn operation_stack(&self) -> EditorOperationStack {
        self.operations().stack()
    }

    pub fn activity_view_descriptor(&self, view_id: &str) -> Option<ActivityViewDescriptor> {
        self.shell()
            .lock()
            .control_service
            .activity_view(view_id)
            .cloned()
    }

    pub fn component_drawer_descriptor(
        &self,
        component_type: &str,
    ) -> Option<ComponentDrawerDescriptor> {
        let inner = self.shell().lock();
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
        let inner = self.shell().lock();
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
        let inner = self.shell().lock();
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
        let inner = self.shell().lock();
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
