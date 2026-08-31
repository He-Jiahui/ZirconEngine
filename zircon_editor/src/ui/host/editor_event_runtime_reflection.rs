use std::sync::OnceLock;

use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventRecord, EditorEventSource, EditorEventTransient,
    EditorViewportEvent, ViewInstanceId,
};
use crate::core::editor_message::{
    EditorMessage, EditorMessageSchemaId, EditorTopic, EditorUiDeltaBarrierKind,
    EditorViewInvalidationMask, EditorViewRefreshReport,
};
use crate::core::extension::{CapabilitySet, FieldEditorContainer, InspectorCustomizationChain};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};
use crate::ui::control::EditorUiControlService;
use crate::ui::host::command_eval_projection::command_eval_ctx_from_chrome;
use crate::ui::host::editor_activity_log::activity_log_console_output_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, apply_transient_projection, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;
use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiReflectionNodePatch};

const WORKBENCH_ROOT_VIEW_INSTANCE_ID: &str = "workbench.root";
const VIEW_INVALIDATED_TOPIC: &str = "view.invalidated";

fn debug_text_schema_id() -> &'static EditorMessageSchemaId {
    static SCHEMA_ID: OnceLock<EditorMessageSchemaId> = OnceLock::new();
    SCHEMA_ID.get_or_init(|| {
        EditorMessageSchemaId::editor("debug-text")
            .expect("the built-in debug-text schema id is valid")
    })
}

impl EditorHostEventController {
    pub(crate) fn refresh_reflection(&self) {
        let mut shell = self.shell().lock();
        let commands = self.commands().lock();
        let i18n = self.context().i18n();
        let locale = i18n.active_locale();
        Self::refresh_reflection_for_shell(
            &mut shell,
            &commands,
            i18n,
            &locale,
            self.context().command_eval(),
            self.play_sessions().mode(),
        );
    }

    pub(crate) fn refresh_reflection_for_shell(
        shell: &mut WorkbenchShellStateData,
        commands: &crate::core::commands::EditorCommandRegistry,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        command_eval: &crate::core::commands::CommandEvalSnapshotHandle,
        play_mode: crate::core::play::PlayModeKind,
    ) {
        let descriptors = shell.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut shell.control_service, views, windows);

        let chrome = Self::build_chrome_for_shell(shell, descriptors);
        let enabled_capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let contribution_capabilities = enabled_capabilities
            .iter()
            .cloned()
            .collect::<CapabilitySet>();
        let contributions = shell.contributions.snapshot();
        let focused_toolkit = shell.manager.focused_document_toolkit();
        shell
            .state
            .viewport_controller
            .set_viewport_overlay_capabilities(&enabled_capabilities);
        let eval_context = shell
            .state
            .project_command_eval_ctx(command_eval_ctx_from_chrome(
                &chrome,
                play_mode,
                enabled_capabilities.clone(),
            ));
        command_eval.replace(eval_context.clone());
        let keymap = shell.manager.keymap();
        let view_model = WorkbenchViewModel::build_with_contributions_and_context(
            commands,
            &keymap,
            i18n,
            locale,
            &chrome,
            &contributions,
            &contribution_capabilities,
            focused_toolkit.as_ref(),
            &eval_context,
        );
        let model = register_workbench_reflection_routes(
            &mut shell.control_service,
            build_workbench_reflection_model(&chrome, &view_model),
        );
        let mut snapshot = crate::ui::EditorUiReflectionAdapter::build_snapshot(&model);
        apply_transient_projection(&mut snapshot, &shell.transient);
        shell.control_service.publish_snapshot(snapshot);
    }

    pub fn refresh_view(
        &self,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> EditorViewRefreshReport {
        self.publish_view_invalidation(view, mask);
        self.drain_pending_view_refreshes()
    }

    fn publish_view_invalidation(&self, view: ViewInstanceId, mask: EditorViewInvalidationMask) {
        if mask.is_empty() {
            return;
        }
        let message = EditorMessage::custom(
            debug_text_schema_id().clone(),
            serde_json::Value::String(view.0.clone()),
        )
        .with_dirty(view.clone(), mask);
        if let Ok(topic) = EditorTopic::parse(VIEW_INVALIDATED_TOPIC) {
            self.context().bus().publish(topic, message);
        } else {
            self.context().bus().mark_view_dirty(view, mask);
        }
    }

    pub fn drain_pending_view_refreshes(&self) -> EditorViewRefreshReport {
        let (dirty, deltas) = self.context().bus().drain_view_updates();
        let has_structure_update = dirty
            .iter()
            .any(|(_, mask)| mask.contains(EditorViewInvalidationMask::TREE_STRUCTURE));
        let mut used_full_snapshot_fallback = dirty
            .iter()
            .any(|(_, mask)| mask != EditorViewInvalidationMask::TREE_STRUCTURE);
        if has_structure_update {
            self.publish_scene_inspection_publication();
        }
        if used_full_snapshot_fallback {
            self.refresh_reflection();
        }
        let patches = deltas.reflection_patches();
        if !patches.is_empty() {
            let patch_result = self
                .shell()
                .lock()
                .control_service
                .apply_reflection_patches(&patches);
            if patch_result.is_err() && !used_full_snapshot_fallback {
                self.refresh_reflection();
                used_full_snapshot_fallback = true;
                let _ = self
                    .shell()
                    .lock()
                    .control_service
                    .apply_reflection_patches(&patches);
            }
        }
        EditorViewRefreshReport::new(dirty, deltas, used_full_snapshot_fallback)
    }

    pub(crate) fn refresh_workbench(&self, mask: EditorViewInvalidationMask) {
        self.refresh_view(ViewInstanceId::new(WORKBENCH_ROOT_VIEW_INSTANCE_ID), mask);
    }

    pub(crate) fn refresh_workbench_for_event_record(&self, record: &EditorEventRecord) {
        let mask = invalidation_mask_for_effects(&record.effects);
        if !matches!(record.source, EditorEventSource::RetainedHost) {
            if !mask.is_empty() {
                self.refresh_workbench(mask);
            }
            return;
        }

        let bus = self.context().bus();
        let patch = reflection_patch_for_event(record);
        if let Some(patch) = patch.as_ref() {
            bus.push_editor_ui_patch(
                ViewInstanceId::new(WORKBENCH_ROOT_VIEW_INSTANCE_ID),
                patch.clone(),
            );
        }
        if let Some(barrier) = delta_barrier_for_event(record) {
            bus.push_editor_ui_barrier(barrier, record.sequence);
        }

        let reflection_changed = record
            .effects
            .contains(&EditorEventEffect::ReflectionChanged);
        let layout_changed = record.effects.contains(&EditorEventEffect::LayoutChanged);
        if layout_changed || (reflection_changed && patch.is_none()) {
            // Events such as focus and drag also clear the previously active node. Until their
            // records carry that previous identity, preserve correctness with one deferred rebuild.
            self.publish_view_invalidation(
                ViewInstanceId::new(WORKBENCH_ROOT_VIEW_INSTANCE_ID),
                EditorViewInvalidationMask::PRESENTATION_DATA,
            );
        }
    }

    pub(crate) fn build_chrome_for_shell(
        shell: &mut WorkbenchShellStateData,
        descriptors: Vec<ViewDescriptor>,
    ) -> EditorChromeSnapshot {
        let inspector_customizations = Self::active_inspector_customizations_for_shell(shell);
        let field_editors = Self::active_field_editors_for_shell(shell);
        let mut editor_snapshot = shell
            .state
            .snapshot_with_inspector_customizations(&inspector_customizations, &field_editors);
        if let Err(error) = Self::project_asset_type_registry_for_shell(shell, &mut editor_snapshot)
        {
            Self::present_asset_type_registry_projection_error(&mut editor_snapshot, error);
        }
        if !shell.state.is_playing() {
            if let Some(history) = shell.manager.focused_animation_history_status() {
                editor_snapshot.can_undo = history.can_undo;
                editor_snapshot.can_redo = history.can_redo;
            }
        }
        editor_snapshot.console_output = activity_log_console_output_for_shell(shell);
        EditorChromeSnapshot::build(
            editor_snapshot,
            &shell.manager.current_layout(),
            shell.manager.current_view_instances(),
            descriptors,
            shell.manager.current_focused_view().as_ref(),
        )
    }

    pub(crate) fn active_inspector_customizations_for_shell(
        shell: &WorkbenchShellStateData,
    ) -> InspectorCustomizationChain {
        let capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .iter()
            .cloned()
            .collect::<CapabilitySet>();
        let mut customizations = InspectorCustomizationChain::default();
        for customization in shell
            .contributions
            .snapshot()
            .inspector_customizations(&capabilities)
        {
            customizations
                .register(customization)
                .expect("contribution store must only retain valid customization ids");
        }
        customizations
    }

    pub(crate) fn active_field_editors_for_shell(
        shell: &WorkbenchShellStateData,
    ) -> FieldEditorContainer {
        let capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .iter()
            .cloned()
            .collect::<CapabilitySet>();
        FieldEditorContainer::with_contributions(
            shell
                .contributions
                .snapshot()
                .field_editors(&capabilities)
                .cloned(),
        )
        .expect("contribution store must only retain valid field editor definitions")
    }
}

fn invalidation_mask_for_effects(effects: &[EditorEventEffect]) -> EditorViewInvalidationMask {
    let mut mask = EditorViewInvalidationMask::NONE;
    for effect in effects {
        match effect {
            EditorEventEffect::LayoutChanged => mask.insert(
                EditorViewInvalidationMask::LAYOUT
                    .union(EditorViewInvalidationMask::PRESENTATION_DATA),
            ),
            EditorEventEffect::RenderChanged => mask.insert(
                EditorViewInvalidationMask::RENDER
                    .union(EditorViewInvalidationMask::PRESENTATION_DATA),
            ),
            EditorEventEffect::PresentationChanged | EditorEventEffect::ReflectionChanged => {
                mask.insert(EditorViewInvalidationMask::PRESENTATION_DATA);
            }
            EditorEventEffect::PresentWelcomeRequested
            | EditorEventEffect::ProjectOpenRequested
            | EditorEventEffect::ProjectSaveRequested
            | EditorEventEffect::DocumentSaveAllRequested
            | EditorEventEffect::ProjectCloseRequested
            | EditorEventEffect::AssetDetailsRefreshRequested
            | EditorEventEffect::AssetPreviewRefreshRequested
            | EditorEventEffect::ImportModelRequested
            | EditorEventEffect::AssetRelocationRequested { .. }
            | EditorEventEffect::AssetDeletionRequested { .. }
            | EditorEventEffect::CommandPaletteOpenRequested
            | EditorEventEffect::SettingsWindowOpenRequested
            | EditorEventEffect::OpenScenePickerRequested
            | EditorEventEffect::CreateScenePickerRequested => {
                mask.insert(EditorViewInvalidationMask::PRESENTATION_DATA);
            }
        }
    }
    mask
}

fn reflection_patch_for_event(record: &EditorEventRecord) -> Option<UiReflectionNodePatch> {
    if record.result.error.is_some()
        || !record
            .effects
            .contains(&EditorEventEffect::ReflectionChanged)
    {
        return None;
    }
    match &record.event {
        EditorEvent::Transient(EditorEventTransient::HoverNode { node_path, hovered }) => Some(
            UiReflectionNodePatch::new(UiNodePath::new(node_path.clone()))
                .with_property("transient.hovered", serde_json::Value::Bool(*hovered)),
        ),
        EditorEvent::Transient(EditorEventTransient::PressNode { node_path, pressed }) => Some(
            UiReflectionNodePatch::new(UiNodePath::new(node_path.clone())).with_pressed(*pressed),
        ),
        EditorEvent::Transient(EditorEventTransient::SetDrawerResizing {
            drawer_id,
            resizing,
        }) => Some(
            UiReflectionNodePatch::new(UiNodePath::new(format!(
                "editor/workbench/drawers/{drawer_id}"
            )))
            .with_property("transient.resizing", serde_json::Value::Bool(*resizing)),
        ),
        _ => None,
    }
}

fn delta_barrier_for_event(record: &EditorEventRecord) -> Option<EditorUiDeltaBarrierKind> {
    if record.result.error.is_some() {
        return None;
    }
    match &record.event {
        EditorEvent::Transient(EditorEventTransient::PressNode { pressed: true, .. })
        | EditorEvent::Viewport(
            EditorViewportEvent::LeftPressed { .. }
            | EditorViewportEvent::RightPressed { .. }
            | EditorViewportEvent::MiddlePressed { .. },
        ) => Some(EditorUiDeltaBarrierKind::Press),
        EditorEvent::Transient(EditorEventTransient::PressNode { pressed: false, .. })
        | EditorEvent::Viewport(
            EditorViewportEvent::LeftReleased
            | EditorViewportEvent::CancelInteraction
            | EditorViewportEvent::RightReleased
            | EditorViewportEvent::MiddleReleased,
        ) => Some(EditorUiDeltaBarrierKind::Release),
        EditorEvent::Viewport(EditorViewportEvent::Scrolled { .. }) => {
            Some(EditorUiDeltaBarrierKind::Scroll)
        }
        EditorEvent::Transient(EditorEventTransient::FocusNode { .. }) => {
            Some(EditorUiDeltaBarrierKind::Focus)
        }
        EditorEvent::Transient(
            EditorEventTransient::SetDrawerResizing { .. }
            | EditorEventTransient::BeginViewDrag { .. }
            | EditorEventTransient::EndViewDrag,
        )
        | EditorEvent::Viewport(EditorViewportEvent::Resized { .. })
        | EditorEvent::Layout(_) => Some(EditorUiDeltaBarrierKind::Geometry),
        _ if record.transaction_id.is_some() || record.save_generation.is_some() => {
            Some(EditorUiDeltaBarrierKind::Commit)
        }
        _ => None,
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
