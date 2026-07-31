use zircon_runtime_interface::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceRecord,
};

use crate::core::asset::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetTypeDefinition,
    AssetTypeId, AssetTypeRegistry,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEnvelope, EditorEventJournal, EditorEventRecord,
    EditorEventSource,
};
use crate::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, EditorUiTemplateDescriptor,
    EditorUiTemplatePaneDataSnapshot,
};
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationSource};
use crate::core::jobs::EditorJobProgressSnapshot;
use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot};
use crate::ui::activity::ActivityViewDescriptor;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};
use crate::ui::host::editor_extension_registration::enabled_asset_types_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    AssetOperationProjectionSnapshot, AssetTypeProjectionSnapshot, AssetWorkspaceSnapshot,
    EditorChromeSnapshot, EditorDataSnapshot, StatusTaskProgressSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};
use std::collections::BTreeMap;
use std::sync::Arc;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEventEnvelope,
};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiInputDispatchResult, UiInputEvent, UiKeyboardInputEvent,
};

impl EditorHostEventController {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        let mut inner = self.shell().lock();
        let component_drawers = Self::active_component_drawers_for_shell(&inner);
        let mut snapshot = inner
            .state
            .snapshot_with_component_drawers(&component_drawers);
        Self::project_asset_type_registry_for_shell(&mut inner, &mut snapshot);
        snapshot
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
        let mut inner = self.shell().lock();
        let descriptors = inner.manager.descriptors();
        Self::build_chrome_for_shell(&mut inner, descriptors)
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

    pub fn job_progress_snapshot(&self) -> Vec<EditorJobProgressSnapshot> {
        self.context().jobs().progress().snapshot()
    }

    pub fn primary_job_progress_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.context().jobs().progress().primary_snapshot()
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
        let Some(command_id) = self.keymap().resolve_keyboard_input(keyboard) else {
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

    pub fn sync_asset_catalog(&self, catalog: Arc<EditorAssetCatalogGeneration>) {
        self.shell().lock().state.sync_asset_catalog(catalog);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_resources(&self, resources: Vec<ResourceRecord>) {
        self.shell().lock().state.sync_asset_resources(resources);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn sync_asset_details(&self, details: Option<Arc<EditorAssetDetailsGeneration>>) {
        self.shell().lock().state.sync_asset_details(details);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn replace_world(
        &self,
        world: impl Into<AuthoringWorldSeed>,
        project_path: impl Into<String>,
    ) -> Result<(), String> {
        self.shell()
            .lock()
            .state
            .replace_world(world, project_path)?;
        self.publish_scene_inspection_resync();
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(())
    }

    pub fn clear_project(&self, welcome: WelcomePaneSnapshot) -> Result<(), String> {
        self.shell().lock().state.clear_project(welcome)?;
        self.publish_scene_inspection_resync();
        self.refresh_workbench(
            EditorViewInvalidationMask::RENDER.union(EditorViewInvalidationMask::PRESENTATION_DATA),
        );
        Ok(())
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

    pub(crate) fn plugin_template_revision(&self) -> (u64, Vec<String>) {
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        (inner.editor_template_generation, enabled_capabilities)
    }

    pub(crate) fn enabled_plugin_template_descriptors(
        &self,
    ) -> (
        u64,
        Vec<String>,
        BTreeMap<String, Vec<EditorUiTemplateDescriptor>>,
    ) {
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let templates_by_owner = inner
            .editor_extensions
            .iter()
            .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
            .fold(BTreeMap::new(), |mut templates_by_owner, registration| {
                templates_by_owner
                    .entry(registration.owner_id().to_owned())
                    .or_default()
                    .extend(registration.registry().ui_templates().into_iter().cloned());
                templates_by_owner
            });
        (
            inner.editor_template_generation,
            enabled_capabilities,
            templates_by_owner,
        )
    }

    pub(crate) fn ui_template_pane_data_snapshots(
        &self,
    ) -> BTreeMap<String, EditorUiTemplatePaneDataSnapshot> {
        let sources = {
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
                .flat_map(|registration| registration.registry().ui_template_pane_data_sources())
                .collect::<BTreeMap<_, _>>()
        };

        sources
            .into_iter()
            .map(|(template_id, source)| (template_id, source.snapshot()))
            .collect()
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

    pub fn asset_type_definition(&self, asset_type: &AssetTypeId) -> Option<AssetTypeDefinition> {
        let mut inner = self.shell().lock();
        enabled_asset_types_for_shell(&mut inner)
            .ok()?
            .get(asset_type)
            .cloned()
    }

    pub(crate) fn project_asset_type_registry_for_shell(
        shell: &mut crate::ui::workbench::shell_state::WorkbenchShellStateData,
        snapshot: &mut EditorDataSnapshot,
    ) {
        let Ok(registry) = enabled_asset_types_for_shell(shell) else {
            return;
        };
        project_asset_workspace(&mut snapshot.asset_activity, &registry);
        project_asset_workspace(&mut snapshot.asset_browser, &registry);
    }

    #[cfg(test)]
    pub(crate) fn asset_type_registry_cache_counts(&self) -> (u64, u64) {
        self.shell().lock().asset_type_registry_cache.counts()
    }

    pub fn asset_creation_templates(
        &self,
        asset_type: &AssetTypeId,
    ) -> Vec<AssetCreationTemplateDescriptor> {
        self.asset_type_definition(asset_type)
            .map(|definition| definition.creation_templates().to_vec())
            .unwrap_or_default()
    }

    pub fn asset_context_commands(
        &self,
        asset_type: &AssetTypeId,
    ) -> Vec<AssetContextCommandDescriptor> {
        self.asset_type_definition(asset_type)
            .map(|definition| definition.context_commands().to_vec())
            .unwrap_or_default()
    }

    pub fn invoke_asset_creation_template(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        template_id: &str,
        target_folder: &str,
    ) -> Result<EditorEventRecord, String> {
        let template = self
            .asset_creation_templates(asset_type)
            .into_iter()
            .find(|template| template.id() == template_id)
            .ok_or_else(|| {
                format!(
                    "asset creation template `{template_id}` is not registered for `{asset_type}`"
                )
            })?;
        self.invoke_operation(
            source,
            EditorOperationInvocation::new(template.operation().clone()).with_arguments(
                serde_json::json!({
                    "asset_type": asset_type.as_str(),
                    "template_id": template.id(),
                    "target_folder": target_folder,
                    "default_document": template.default_document(),
                }),
            ),
        )
    }

    pub fn invoke_asset_context_command(
        &self,
        source: EditorOperationSource,
        asset_type: &AssetTypeId,
        command_id: &str,
        asset_locator: &str,
    ) -> Result<EditorEventRecord, String> {
        let command = self
            .asset_context_commands(asset_type)
            .into_iter()
            .find(|command| command.id() == command_id)
            .ok_or_else(|| {
                format!("asset context command `{command_id}` is not registered for `{asset_type}`")
            })?;
        self.invoke_operation(
            source,
            EditorOperationInvocation::new(command.operation().clone()).with_arguments(
                serde_json::json!({
                    "asset_type": asset_type.as_str(),
                    "command_id": command.id(),
                    "asset_locator": asset_locator,
                }),
            ),
        )
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

fn project_asset_workspace(snapshot: &mut AssetWorkspaceSnapshot, registry: &AssetTypeRegistry) {
    for item in &mut snapshot.visible_assets {
        let Ok(asset_type) = AssetTypeId::parse(item.asset_type.asset_type_id.clone()) else {
            continue;
        };
        if let Some(definition) = registry.get(&asset_type) {
            item.asset_type = AssetTypeProjectionSnapshot::from_definition(definition);
        }
    }

    snapshot.creation_templates = registry
        .definitions()
        .flat_map(|definition| {
            definition.creation_templates().iter().map(|template| {
                AssetOperationProjectionSnapshot {
                    asset_type_id: definition.id().to_string(),
                    id: template.id().to_owned(),
                    display_name: template.display_name().to_owned(),
                    operation_id: template.operation().to_string(),
                    icon_name: None,
                    default_document: template.default_document().map(str::to_owned),
                }
            })
        })
        .collect();

    let selection_type = snapshot.selection.asset_type.asset_type_id.clone();
    let Ok(asset_type) = AssetTypeId::parse(selection_type) else {
        return;
    };
    let Some(definition) = registry.get(&asset_type) else {
        return;
    };
    snapshot.selection.asset_type = AssetTypeProjectionSnapshot::from_definition(definition);
    snapshot.selection.toolkit_view_id = definition
        .toolkit()
        .map(|toolkit| toolkit.view_id().to_owned())
        .unwrap_or_default();
    snapshot.selection.toolkit_open_operation = definition
        .toolkit()
        .map(|toolkit| toolkit.open_operation().to_string())
        .unwrap_or_default();
    snapshot.selection.context_commands = definition
        .context_commands()
        .iter()
        .map(|command| AssetOperationProjectionSnapshot {
            asset_type_id: definition.id().to_string(),
            id: command.id().to_owned(),
            display_name: command.display_name().to_owned(),
            operation_id: command.operation().to_string(),
            icon_name: command.icon_name().map(str::to_owned),
            default_document: None,
        })
        .collect();
    for reference in snapshot
        .selection
        .references
        .iter_mut()
        .chain(snapshot.selection.used_by.iter_mut())
    {
        reference.asset_type = reference
            .kind
            .and_then(|kind| projected_asset_type_for_kind(registry, kind));
    }
    for subasset in &mut snapshot.selection.subassets {
        if let Some(asset_type) = projected_asset_type_for_kind(registry, subasset.kind) {
            subasset.asset_type = asset_type;
        }
    }
}

fn projected_asset_type_for_kind(
    registry: &AssetTypeRegistry,
    kind: zircon_runtime_interface::resource::ResourceKind,
) -> Option<AssetTypeProjectionSnapshot> {
    registry
        .get(&AssetTypeId::from_resource_kind(kind))
        .map(AssetTypeProjectionSnapshot::from_definition)
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn keyboard_dispatch_reuses_the_controller_keymap() {
        let source = include_str!("editor_event_runtime_access.rs");
        let reparsing = ["EditorKeymap::", "default_workbench()"].concat();

        assert!(!source.contains(&reparsing));
    }
}
