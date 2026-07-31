use std::collections::BTreeMap;

use crate::core::editor_event::{EditorEventEffect, ViewInstanceId};
use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};
use crate::core::editor_message::{
    EditorMessage, EditorTopic, EditorViewInvalidationMask, EditorViewRefreshReport,
};
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};
use crate::ui::control::EditorUiControlService;
use crate::ui::host::EditorHostEventController;
use crate::ui::host::command_eval_projection::command_eval_ctx_from_chrome;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, apply_transient_projection, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

const WORKBENCH_ROOT_VIEW_INSTANCE_ID: &str = "workbench.root";
const VIEW_INVALIDATED_TOPIC: &str = "view.invalidated";

impl EditorHostEventController {
    pub(crate) fn refresh_reflection(&self) {
        let mut shell = self.shell().lock();
        let commands = self.commands().lock();
        Self::refresh_reflection_for_shell(
            &mut shell,
            &commands,
            self.context().command_eval(),
            self.play_sessions().mode(),
        );
    }

    pub(crate) fn refresh_reflection_for_shell(
        shell: &mut WorkbenchShellStateData,
        commands: &crate::core::commands::EditorCommandRegistry,
        command_eval: &crate::core::commands::CommandEvalSnapshotHandle,
        play_mode: crate::core::play::PlayModeKind,
    ) {
        let descriptors = shell.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut shell.control_service, views, windows);

        let chrome = Self::build_chrome_for_shell(shell, descriptors);
        let active_extensions = active_extension_registries(shell);
        let enabled_capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
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
        let view_model = WorkbenchViewModel::build_with_extensions_and_context(
            commands,
            &chrome,
            &active_extensions,
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
        let message = EditorMessage::custom(
            "zircon.editor.debug-text",
            serde_json::Value::String(view.0.clone()),
        )
        .with_dirty(view.clone(), mask);
        if let Ok(topic) = EditorTopic::parse(VIEW_INVALIDATED_TOPIC) {
            self.context().bus().publish(topic, message);
        } else {
            self.context().bus().mark_view_dirty(view, mask);
        }
        self.drain_pending_view_refreshes()
    }

    pub fn drain_pending_view_refreshes(&self) -> EditorViewRefreshReport {
        let dirty = self.context().bus().drain_dirty();
        let used_full_snapshot_fallback = !dirty.is_empty();
        if used_full_snapshot_fallback {
            self.refresh_reflection();
        }
        EditorViewRefreshReport::new(dirty, used_full_snapshot_fallback)
    }

    pub(crate) fn refresh_workbench(&self, mask: EditorViewInvalidationMask) {
        self.refresh_view(ViewInstanceId::new(WORKBENCH_ROOT_VIEW_INSTANCE_ID), mask);
    }

    pub(crate) fn refresh_workbench_for_effects(&self, effects: &[EditorEventEffect]) {
        self.refresh_workbench(invalidation_mask_for_effects(effects));
    }

    pub(crate) fn build_chrome_for_shell(
        shell: &mut WorkbenchShellStateData,
        descriptors: Vec<ViewDescriptor>,
    ) -> EditorChromeSnapshot {
        let component_drawers = Self::active_component_drawers_for_shell(shell);
        let mut editor_snapshot = shell
            .state
            .snapshot_with_component_drawers(&component_drawers);
        Self::project_asset_type_registry_for_shell(shell, &mut editor_snapshot);
        EditorChromeSnapshot::build(
            editor_snapshot,
            &shell.manager.current_layout(),
            shell.manager.current_view_instances(),
            descriptors,
            shell.manager.current_focused_view().as_ref(),
        )
    }

    pub(crate) fn active_component_drawers_for_shell(
        shell: &WorkbenchShellStateData,
    ) -> BTreeMap<String, ComponentDrawerDescriptor> {
        active_extension_registries(shell)
            .into_iter()
            .flat_map(|registry| {
                registry
                    .component_drawers()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .map(|descriptor| (descriptor.component_type().to_string(), descriptor))
            .collect()
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
            | EditorEventEffect::ProjectCloseRequested
            | EditorEventEffect::AssetDetailsRefreshRequested
            | EditorEventEffect::AssetPreviewRefreshRequested
            | EditorEventEffect::ImportModelRequested
            | EditorEventEffect::CommandPaletteOpenRequested => {
                mask.insert(EditorViewInvalidationMask::PRESENTATION_DATA);
            }
        }
    }
    if mask.is_empty() {
        EditorViewInvalidationMask::PRESENTATION_DATA
    } else {
        mask
    }
}

fn active_extension_registries(shell: &WorkbenchShellStateData) -> Vec<EditorExtensionRegistry> {
    let enabled_capabilities = shell
        .manager
        .capability_snapshot()
        .enabled_capabilities()
        .to_vec();
    shell
        .editor_extensions
        .iter()
        .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
        .map(|registration| registration.registry().clone())
        .collect()
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
