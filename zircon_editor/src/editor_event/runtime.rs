use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use zircon_editor_ui::{
    ActivityViewDescriptor, ActivityWindowDescriptor, EditorUiBinding, EditorUiBindingPayload,
    EditorUiControlService, EditorUiReflectionAdapter,
};
use zircon_manager::{
    AssetRecordKind, EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
    ResourceStatusRecord,
};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::LevelSystem;
use zircon_ui::{
    UiControlRequest, UiControlResponse, UiEventBinding, UiInvocationError, UiInvocationResult,
    UiNodeDescriptor, UiNodePath, UiPropertyDescriptor, UiReflectionSnapshot, UiRouteId,
    UiValueType,
};

use crate::{
    activity_descriptors_from_views, apply_inspector_binding, build_workbench_reflection_model,
    dispatch_asset_binding, dispatch_docking_binding, dispatch_inspector_binding,
    dispatch_selection_binding, dispatch_viewport_binding, dispatch_workbench_binding,
    register_workbench_reflection_routes, EditorChromeSnapshot, EditorDataSnapshot, EditorIntent,
    EditorManager, EditorState, LayoutCommand, MenuAction, ViewDescriptor, ViewDescriptorId,
    WorkbenchViewModel,
};
use crate::snapshot::{
    AssetUtilityTab as SnapshotAssetUtilityTab, AssetViewMode as SnapshotAssetViewMode,
};
use crate::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode, EditorEvent,
    EditorEventEffect, EditorEventEnvelope, EditorEventId, EditorEventJournal, EditorEventRecord,
    EditorEventResult, EditorEventSequence, EditorEventSource, EditorEventUndoPolicy,
    EditorInspectorEvent, EditorTransientUiState, EditorViewportEvent,
};

pub trait EditorEventDispatcher {
    fn dispatch_envelope(&self, envelope: EditorEventEnvelope) -> Result<EditorEventRecord, String>;
    fn dispatch_binding(
        &self,
        binding: EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String>;
    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String>;
}

pub struct EditorEventRuntime {
    inner: Mutex<EditorEventRuntimeInner>,
}

struct EditorEventRuntimeInner {
    state: EditorState,
    manager: Arc<EditorManager>,
    transient: EditorTransientUiState,
    journal: EditorEventJournal,
    control_service: EditorUiControlService,
    next_event_id: u64,
    next_sequence: u64,
    revision: u64,
    dragging_gizmo: bool,
}

struct ExecutionOutcome {
    changed: bool,
    effects: Vec<EditorEventEffect>,
}

impl EditorEventRuntime {
    pub fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        let runtime = Self {
            inner: Mutex::new(EditorEventRuntimeInner {
                state,
                manager,
                transient: EditorTransientUiState::default(),
                journal: EditorEventJournal::default(),
                control_service: EditorUiControlService::default(),
                next_event_id: 0,
                next_sequence: 0,
                revision: 0,
                dragging_gizmo: false,
            }),
        };
        runtime.refresh_reflection();
        runtime
    }

    pub fn handle_control_request(&self, request: UiControlRequest) -> UiControlResponse {
        match request {
            UiControlRequest::InvokeBinding { binding } => {
                UiControlResponse::Invocation(self.invoke_binding(binding))
            }
            UiControlRequest::InvokeRoute {
                route_id,
                arguments,
            } => UiControlResponse::Invocation(self.invoke_route(route_id, arguments)),
            UiControlRequest::CallAction {
                node_path,
                action_id,
                arguments,
            } => UiControlResponse::Invocation(self.call_action(node_path, action_id, arguments)),
            other => {
                let mut inner = self.inner.lock().unwrap();
                inner.control_service.handle_request(other)
            }
        }
    }

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

    pub fn viewport_state(&self) -> zircon_graphics::ViewportState {
        self.inner.lock().unwrap().state.viewport_state()
    }

    pub fn set_status_line(&self, message: impl Into<String>) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.set_status_line(message);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn update_name_field(&self, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.update_name_field(value);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn update_parent_field(&self, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.update_parent_field(value);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn update_translation_field(&self, axis: usize, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.update_translation_field(axis, value);
        Self::refresh_reflection_locked(&mut inner);
    }

    pub fn set_mesh_import_path(&self, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.set_mesh_import_path(value);
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

    pub fn sync_asset_resources(&self, resources: Vec<ResourceStatusRecord>) {
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
        binding: EditorUiBinding,
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

    fn invoke_binding(&self, binding: UiEventBinding) -> UiInvocationResult {
        let route_id = {
            let inner = self.inner.lock().unwrap();
            inner.control_service.route_id_for_binding(&binding)
        };
        let editor_binding = match EditorUiBinding::from_ui_binding(binding.clone()) {
            Ok(binding) => binding,
            Err(error) => {
                return UiInvocationResult {
                    route_id,
                    binding: Some(binding),
                    value: None,
                    error: Some(UiInvocationError::HandlerFailed(error.to_string())),
                };
            }
        };
        self.invoke_editor_binding(route_id, editor_binding)
    }

    fn invoke_route(
        &self,
        route_id: UiRouteId,
        arguments: Vec<zircon_ui::UiBindingValue>,
    ) -> UiInvocationResult {
        let binding = {
            let inner = self.inner.lock().unwrap();
            inner.control_service.route_binding(route_id)
        };
        let Some(binding) = binding else {
            return UiInvocationResult::failure(
                Some(route_id),
                None,
                UiInvocationError::UnknownRoute(route_id),
            );
        };

        let editor_binding = match EditorUiBinding::from_ui_binding(binding.clone()) {
            Ok(binding) => binding,
            Err(error) => {
                return UiInvocationResult::failure(
                    Some(route_id),
                    Some(binding),
                    UiInvocationError::HandlerFailed(error.to_string()),
                );
            }
        };
        let editor_binding = if arguments.is_empty() {
            editor_binding
        } else {
            match editor_binding.with_arguments(arguments) {
                Ok(binding) => binding,
                Err(error) => {
                    return UiInvocationResult::failure(
                        Some(route_id),
                        Some(binding),
                        UiInvocationError::HandlerFailed(error.to_string()),
                    );
                }
            }
        };

        self.invoke_editor_binding(Some(route_id), editor_binding)
    }

    fn call_action(
        &self,
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<zircon_ui::UiBindingValue>,
    ) -> UiInvocationResult {
        let route_id = {
            let inner = self.inner.lock().unwrap();
            let Some(node) = inner.control_service.query_node(&node_path) else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::UnknownNode(node_path.0),
                );
            };
            let Some(action) = node.actions.get(&action_id) else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::UnknownAction {
                        node_path: node.node_path.0,
                        action_id,
                    },
                );
            };
            if !action.callable_from_remote {
                return UiInvocationResult::failure(
                    action.route_id,
                    None,
                    UiInvocationError::ActionNotCallable {
                        node_path: node_path.0,
                        action_id: action.action_id.clone(),
                    },
                );
            }
            let Some(route_id) = action.route_id else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::ActionMissingRoute {
                        node_path: node_path.0,
                        action_id: action.action_id.clone(),
                    },
                );
            };
            route_id
        };

        self.invoke_route(route_id, arguments)
    }

    fn invoke_editor_binding(
        &self,
        route_id: Option<UiRouteId>,
        binding: EditorUiBinding,
    ) -> UiInvocationResult {
        let ui_binding = binding.as_ui_binding();
        match self.dispatch_binding(binding, EditorEventSource::Headless) {
            Ok(record) => UiInvocationResult {
                route_id,
                binding: Some(ui_binding),
                value: record.result.value,
                error: None,
            },
            Err(error) => UiInvocationResult {
                route_id,
                binding: Some(ui_binding),
                value: None,
                error: Some(UiInvocationError::HandlerFailed(error)),
            },
        }
    }

    fn refresh_reflection(&self) {
        let mut inner = self.inner.lock().unwrap();
        Self::refresh_reflection_locked(&mut inner);
    }

    fn refresh_reflection_locked(inner: &mut EditorEventRuntimeInner) {
        let descriptors = inner.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut inner.control_service, views, windows);

        let chrome = Self::build_chrome_locked(inner, descriptors);
        let view_model = WorkbenchViewModel::build(&chrome);
        let model = register_workbench_reflection_routes(
            &mut inner.control_service,
            build_workbench_reflection_model(&chrome, &view_model),
        );
        let mut snapshot = EditorUiReflectionAdapter::build_snapshot(&model);
        apply_transient_projection(&mut snapshot, &inner.transient);
        inner.control_service.publish_snapshot(snapshot);
    }

    fn build_chrome_locked(
        inner: &EditorEventRuntimeInner,
        descriptors: Vec<ViewDescriptor>,
    ) -> EditorChromeSnapshot {
        EditorChromeSnapshot::build(
            inner.state.snapshot(),
            &inner.manager.current_layout(),
            inner.manager.current_view_instances(),
            descriptors,
        )
    }

    fn dispatch_normalized_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        let mut inner = self.inner.lock().unwrap();
        inner.next_event_id += 1;
        inner.next_sequence += 1;

        let before_revision = inner.revision;
        let after_revision = before_revision + 1;
        inner.revision = after_revision;

        let event_id = EditorEventId::new(inner.next_event_id);
        let sequence = EditorEventSequence::new(inner.next_sequence);
        let undo_policy = undo_policy_for_event(&event);

        let execution = match execute_event(&mut inner, &event) {
            Ok(outcome) => outcome,
            Err(error) => {
                inner.state.set_status_line(error.clone());
                let record = EditorEventRecord {
                    event_id,
                    sequence,
                    source,
                    event,
                    effects: vec![
                        EditorEventEffect::PresentationChanged,
                        EditorEventEffect::ReflectionChanged,
                    ],
                    undo_policy,
                    before_revision,
                    after_revision,
                    result: EditorEventResult::failure(error.clone()),
                };
                Self::refresh_reflection_locked(&mut inner);
                inner.journal.push(record.clone());
                return Err(error);
            }
        };

        let record = EditorEventRecord {
            event_id,
            sequence,
            source,
            event,
            effects: execution.effects.clone(),
            undo_policy,
            before_revision,
            after_revision,
            result: EditorEventResult::success(event_result_value(
                after_revision,
                execution.changed,
            )),
        };
        Self::refresh_reflection_locked(&mut inner);
        inner.journal.push(record.clone());
        Ok(record)
    }
}

impl EditorEventDispatcher for EditorEventRuntime {
    fn dispatch_envelope(&self, envelope: EditorEventEnvelope) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event(envelope.source, envelope.event)
    }

    fn dispatch_binding(
        &self,
        binding: EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let event = normalize_binding(&binding)?;
        self.dispatch_normalized_event(source, event)
    }

    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event(source, event)
    }
}

fn register_activity_descriptors(
    service: &mut EditorUiControlService,
    views: Vec<ActivityViewDescriptor>,
    windows: Vec<ActivityWindowDescriptor>,
) {
    for descriptor in views {
        if service.activity_view(&descriptor.view_id).is_none() {
            let _ = service.register_activity_view(descriptor);
        }
    }
    for descriptor in windows {
        if service.activity_window(&descriptor.window_id).is_none() {
            let _ = service.register_activity_window(descriptor);
        }
    }
}

fn normalize_binding(binding: &EditorUiBinding) -> Result<EditorEvent, String> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { .. } => {
            let crate::WorkbenchHostEvent::Menu(action) =
                dispatch_workbench_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::WorkbenchMenu(action))
        }
        EditorUiBindingPayload::DockCommand(_) => Ok(EditorEvent::Layout(
            dispatch_docking_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::SelectionCommand(_) => Ok(EditorEvent::Selection(
            dispatch_selection_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::AssetCommand(_) => {
            let event = dispatch_asset_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Asset(match event {
                crate::AssetHostEvent::OpenAsset { asset_path } => {
                    EditorAssetEvent::OpenAsset { asset_path }
                }
            }))
        }
        EditorUiBindingPayload::InspectorFieldBatch { .. } => {
            let batch = dispatch_inspector_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Inspector(EditorInspectorEvent {
                subject_path: batch.subject_path,
                changes: batch.changes,
            }))
        }
        EditorUiBindingPayload::ViewportCommand(_) => Ok(EditorEvent::Viewport(
            viewport_event_from_binding(binding)?,
        )),
        EditorUiBindingPayload::PositionOfTrackAndFrame { .. }
        | EditorUiBindingPayload::Custom(_) => Err(format!(
            "unsupported editor event binding {}",
            binding.native_binding()
        )),
    }
}

fn viewport_event_from_binding(binding: &EditorUiBinding) -> Result<EditorViewportEvent, String> {
    Ok(match dispatch_viewport_binding(binding).map_err(|error| error.to_string())? {
        zircon_graphics::ViewportInput::PointerMoved(position) => {
            EditorViewportEvent::PointerMoved {
                x: position.x,
                y: position.y,
            }
        }
        zircon_graphics::ViewportInput::LeftPressed(position) => EditorViewportEvent::LeftPressed {
            x: position.x,
            y: position.y,
        },
        zircon_graphics::ViewportInput::LeftReleased => EditorViewportEvent::LeftReleased,
        zircon_graphics::ViewportInput::RightPressed(position) => {
            EditorViewportEvent::RightPressed {
                x: position.x,
                y: position.y,
            }
        }
        zircon_graphics::ViewportInput::RightReleased => EditorViewportEvent::RightReleased,
        zircon_graphics::ViewportInput::MiddlePressed(position) => {
            EditorViewportEvent::MiddlePressed {
                x: position.x,
                y: position.y,
            }
        }
        zircon_graphics::ViewportInput::MiddleReleased => EditorViewportEvent::MiddleReleased,
        zircon_graphics::ViewportInput::Scrolled(delta) => {
            EditorViewportEvent::Scrolled { delta }
        }
        zircon_graphics::ViewportInput::Resized(size) => EditorViewportEvent::Resized {
            width: size.x,
            height: size.y,
        },
    })
}

fn execute_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorEvent::WorkbenchMenu(action) => execute_menu_action(inner, action),
        EditorEvent::Layout(command) => execute_layout_command(inner, command),
        EditorEvent::Selection(event) => execute_selection(inner, event),
        EditorEvent::Asset(event) => execute_asset_event(inner, event),
        EditorEvent::Inspector(event) => execute_inspector_event(inner, event),
        EditorEvent::Viewport(event) => execute_viewport_event(inner, event),
        EditorEvent::Transient(update) => {
            inner.transient.apply(update);
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
    }
}

fn execute_menu_action(
    inner: &mut EditorEventRuntimeInner,
    action: &MenuAction,
) -> Result<ExecutionOutcome, String> {
    match action {
        MenuAction::OpenProject => {
            let path = PathBuf::from(inner.state.snapshot().project_path);
            let document = inner
                .manager
                .open_project(&path)
                .map_err(|error| error.to_string())?;
            let level = inner
                .manager
                .create_runtime_level(document.world)
                .map_err(|error| error.to_string())?;
            inner.state.replace_world(
                level,
                document.root_path.to_string_lossy().into_owned(),
            );
            inner
                .manager
                .apply_project_workspace(document.editor_workspace)
                .map_err(|error| error.to_string())?;
            inner
                .state
                .set_status_line(format!("Loaded project from {}", path.display()));
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::ProjectOpenRequested,
                    EditorEventEffect::LayoutChanged,
                    EditorEventEffect::RenderChanged,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::SaveProject => {
            let path = PathBuf::from(inner.state.snapshot().project_path);
            let scene = inner
                .state
                .project_scene()
                .ok_or_else(|| "No project open".to_string())?;
            inner
                .manager
                .save_project(&path, &scene)
                .map_err(|error| error.to_string())?;
            inner.state.mark_project_open();
            inner
                .state
                .set_status_line(format!("Saved project to {}", path.display()));
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::ProjectSaveRequested,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::SaveLayout => {
            inner
                .manager
                .save_global_default_layout()
                .map_err(|error| error.to_string())?;
            inner.state.set_status_line("Saved global default layout");
            Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::ResetLayout => {
            let changed = inner
                .manager
                .apply_layout_command(LayoutCommand::ResetToDefault)
                .map_err(|error| error.to_string())?;
            inner.state.set_status_line("Reset layout");
            Ok(ExecutionOutcome {
                changed,
                effects: vec![
                    EditorEventEffect::LayoutChanged,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::Undo => scene_intent_event(inner, EditorIntent::Undo),
        MenuAction::Redo => scene_intent_event(inner, EditorIntent::Redo),
        MenuAction::CreateNode(kind) => {
            scene_intent_event(inner, EditorIntent::CreateNode(kind.clone()))
        }
        MenuAction::DeleteSelected => {
            let changed = inner.state.delete_selected()?;
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::OpenView(descriptor_id) => {
            let instance_id = inner
                .manager
                .open_view(descriptor_id.clone(), None)
                .map_err(|error| error.to_string())?;
            let focused = inner
                .manager
                .focus_view(&instance_id)
                .map_err(|error| error.to_string())?;
            inner
                .state
                .set_status_line(format!("Opened view {}", descriptor_id.0));
            Ok(ExecutionOutcome {
                changed: focused || !instance_id.0.is_empty(),
                effects: vec![
                    EditorEventEffect::LayoutChanged,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
    }
}

fn execute_layout_command(
    inner: &mut EditorEventRuntimeInner,
    command: &LayoutCommand,
) -> Result<ExecutionOutcome, String> {
    let changed = inner
        .manager
        .apply_layout_command(command.clone())
        .map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn execute_selection(
    inner: &mut EditorEventRuntimeInner,
    event: &crate::SelectionHostEvent,
) -> Result<ExecutionOutcome, String> {
    let changed = match event {
        crate::SelectionHostEvent::SelectSceneNode { node_id } => {
            inner.state.apply_intent(EditorIntent::SelectNode(*node_id))?
        }
    };
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn execute_asset_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorAssetEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorAssetEvent::OpenAsset { asset_path } => {
            inner
                .state
                .set_status_line(format!("Open asset requested for {asset_path}"));
            Ok(asset_effects(false))
        }
        EditorAssetEvent::SelectFolder { folder_id } => {
            inner.state.select_asset_folder(folder_id.clone());
            Ok(asset_effects(true))
        }
        EditorAssetEvent::SelectItem { asset_uuid } => {
            inner.state.select_asset(Some(asset_uuid.clone()));
            Ok(asset_effects(true))
        }
        EditorAssetEvent::ActivateReference { asset_uuid } => {
            inner.state.navigate_to_asset(asset_uuid);
            Ok(asset_effects(true))
        }
        EditorAssetEvent::SetSearchQuery { query } => {
            inner.state.set_asset_search_query(query.clone());
            Ok(asset_effects(true))
        }
        EditorAssetEvent::SetKindFilter { kind } => {
            inner
                .state
                .set_asset_kind_filter(parse_asset_kind_filter(kind.as_deref())?);
            Ok(asset_effects(true))
        }
        EditorAssetEvent::SetViewMode { surface, view_mode } => {
            match (surface, view_mode) {
                (EditorAssetSurface::Activity, EditorAssetViewMode::List) => {
                    inner
                        .state
                        .set_asset_activity_view_mode(SnapshotAssetViewMode::List)
                }
                (EditorAssetSurface::Activity, EditorAssetViewMode::Thumbnail) => inner
                    .state
                    .set_asset_activity_view_mode(SnapshotAssetViewMode::Thumbnail),
                (EditorAssetSurface::Browser, EditorAssetViewMode::List) => {
                    inner
                        .state
                        .set_asset_browser_view_mode(SnapshotAssetViewMode::List)
                }
                (EditorAssetSurface::Browser, EditorAssetViewMode::Thumbnail) => inner
                    .state
                    .set_asset_browser_view_mode(SnapshotAssetViewMode::Thumbnail),
            }
            Ok(asset_effects(true))
        }
        EditorAssetEvent::SetUtilityTab { surface, tab } => {
            let tab = match tab {
                EditorAssetUtilityTab::Preview => SnapshotAssetUtilityTab::Preview,
                EditorAssetUtilityTab::References => SnapshotAssetUtilityTab::References,
                EditorAssetUtilityTab::Metadata => SnapshotAssetUtilityTab::Metadata,
                EditorAssetUtilityTab::Plugins => SnapshotAssetUtilityTab::Plugins,
            };
            match surface {
                EditorAssetSurface::Activity => inner.state.set_asset_activity_tab(tab),
                EditorAssetSurface::Browser => inner.state.set_asset_browser_tab(tab),
            }
            Ok(asset_effects(true))
        }
        EditorAssetEvent::OpenAssetBrowser => {
            open_view(inner, "editor.asset_browser", "Opened asset browser")
        }
        EditorAssetEvent::LocateSelectedAsset => open_view(inner, "editor.assets", "Opened assets"),
    }
}

fn execute_inspector_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorInspectorEvent,
) -> Result<ExecutionOutcome, String> {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        zircon_editor_ui::EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(event.subject_path.clone(), event.changes.clone()),
    );
    let changed =
        apply_inspector_binding(&mut inner.state, &binding).map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}

fn execute_viewport_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, String> {
    let feedback = match event {
        EditorViewportEvent::PointerMoved { x, y } => {
            let feedback = inner.state.handle_viewport_input(
                zircon_graphics::ViewportInput::PointerMoved(zircon_math::Vec2::new(*x, *y)),
            );
            if inner.dragging_gizmo && feedback.transformed_node.is_some() {
                inner.state.apply_intent(EditorIntent::DragGizmo)?;
            }
            feedback
        }
        EditorViewportEvent::LeftPressed { x, y } => {
            let feedback = inner.state.handle_viewport_input(
                zircon_graphics::ViewportInput::LeftPressed(zircon_math::Vec2::new(*x, *y)),
            );
            inner.dragging_gizmo = feedback.hovered_axis.is_some();
            if inner.dragging_gizmo {
                inner.state.apply_intent(EditorIntent::BeginGizmoDrag)?;
            }
            feedback
        }
        EditorViewportEvent::LeftReleased => {
            if inner.dragging_gizmo {
                inner.state.apply_intent(EditorIntent::EndGizmoDrag)?;
            }
            inner.dragging_gizmo = false;
            inner
                .state
                .handle_viewport_input(zircon_graphics::ViewportInput::LeftReleased)
        }
        EditorViewportEvent::RightPressed { x, y } => inner.state.handle_viewport_input(
            zircon_graphics::ViewportInput::RightPressed(zircon_math::Vec2::new(*x, *y)),
        ),
        EditorViewportEvent::RightReleased => inner
            .state
            .handle_viewport_input(zircon_graphics::ViewportInput::RightReleased),
        EditorViewportEvent::MiddlePressed { x, y } => inner.state.handle_viewport_input(
            zircon_graphics::ViewportInput::MiddlePressed(zircon_math::Vec2::new(*x, *y)),
        ),
        EditorViewportEvent::MiddleReleased => inner
            .state
            .handle_viewport_input(zircon_graphics::ViewportInput::MiddleReleased),
        EditorViewportEvent::Scrolled { delta } => inner
            .state
            .handle_viewport_input(zircon_graphics::ViewportInput::Scrolled(*delta)),
        EditorViewportEvent::Resized { width, height } => inner.state.handle_viewport_input(
            zircon_graphics::ViewportInput::Resized(zircon_math::UVec2::new(*width, *height)),
        ),
    };
    Ok(ExecutionOutcome {
        changed: matches!(event, EditorViewportEvent::LeftReleased | EditorViewportEvent::Resized { .. })
            || feedback.camera_updated
            || feedback.transformed_node.is_some()
            || feedback.hovered_axis.is_some(),
        effects: vec![
            EditorEventEffect::RenderChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn scene_intent_event(
    inner: &mut EditorEventRuntimeInner,
    intent: EditorIntent,
) -> Result<ExecutionOutcome, String> {
    let changed = inner.state.apply_intent(intent)?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}

fn scene_effects() -> Vec<EditorEventEffect> {
    vec![
        EditorEventEffect::RenderChanged,
        EditorEventEffect::PresentationChanged,
        EditorEventEffect::ReflectionChanged,
    ]
}

fn asset_effects(changed: bool) -> ExecutionOutcome {
    ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::AssetWorkspaceChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    }
}

fn open_view(
    inner: &mut EditorEventRuntimeInner,
    descriptor_id: &str,
    status_line: &str,
) -> Result<ExecutionOutcome, String> {
    let instance_id = inner
        .manager
        .open_view(ViewDescriptorId::new(descriptor_id), None)
        .map_err(|error| error.to_string())?;
    let focused = inner
        .manager
        .focus_view(&instance_id)
        .map_err(|error| error.to_string())?;
    inner.state.set_status_line(status_line);
    Ok(ExecutionOutcome {
        changed: focused || !instance_id.0.is_empty(),
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn parse_asset_kind_filter(kind: Option<&str>) -> Result<Option<AssetRecordKind>, String> {
    match kind.unwrap_or_default() {
        "" | "All" => Ok(None),
        "Texture" => Ok(Some(AssetRecordKind::Texture)),
        "Shader" => Ok(Some(AssetRecordKind::Shader)),
        "Material" => Ok(Some(AssetRecordKind::Material)),
        "Scene" => Ok(Some(AssetRecordKind::Scene)),
        "Model" => Ok(Some(AssetRecordKind::Model)),
        other => Err(format!("unknown asset kind filter {other}")),
    }
}

fn undo_policy_for_event(event: &EditorEvent) -> EditorEventUndoPolicy {
    match event {
        EditorEvent::WorkbenchMenu(
            MenuAction::CreateNode(_)
                | MenuAction::DeleteSelected
                | MenuAction::Undo
                | MenuAction::Redo,
        )
        | EditorEvent::Inspector(_)
        | EditorEvent::Viewport(_) => EditorEventUndoPolicy::DelegatedToEditorHistory,
        EditorEvent::Layout(_)
        | EditorEvent::Asset(_)
        | EditorEvent::WorkbenchMenu(
            MenuAction::OpenProject
                | MenuAction::SaveProject
                | MenuAction::SaveLayout
                | MenuAction::ResetLayout
                | MenuAction::OpenView(_),
        ) => EditorEventUndoPolicy::FutureInverseEvent,
        EditorEvent::Selection(_) | EditorEvent::Transient(_) => {
            EditorEventUndoPolicy::NonUndoable
        }
    }
}

fn event_result_value(revision: u64, changed: bool) -> Value {
    json!({
        "revision": revision,
        "changed": changed,
    })
}

fn apply_transient_projection(
    snapshot: &mut UiReflectionSnapshot,
    transient: &EditorTransientUiState,
) {
    for node in snapshot.nodes.values_mut() {
        let node_path = node.node_path.0.clone();
        let hovered = transient.is_node_hovered(&node_path);
        let focused = transient.is_node_focused(&node_path);
        let pressed = transient.is_node_pressed(&node_path);
        let resizing = drawer_id_from_path(&node_path)
            .is_some_and(|drawer_id| transient.is_drawer_resizing(drawer_id));
        let dragging = node_path
            .rsplit('/')
            .next()
            .is_some_and(|segment| transient.is_view_dragging(segment));

        node.state_flags.pressed = pressed;
        upsert_property(node, "transient.hovered", hovered);
        upsert_property(node, "transient.focused", focused);
        upsert_property(node, "transient.resizing", resizing);
        upsert_property(node, "transient.dragging", dragging);
    }
}

fn upsert_property(node: &mut UiNodeDescriptor, name: &str, value: bool) {
    node.properties.insert(
        name.to_string(),
        UiPropertyDescriptor::new(name, UiValueType::Bool, json!(value)),
    );
}

fn drawer_id_from_path(node_path: &str) -> Option<&str> {
    let prefix = "editor/workbench/drawers/";
    let remainder = node_path.strip_prefix(prefix)?;
    remainder.split('/').next()
}
