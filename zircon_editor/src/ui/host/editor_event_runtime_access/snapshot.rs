use crate::core::editing::engine::EditCommandError;
use crate::core::play::WorldDomain;
use std::collections::BTreeMap;

use crate::scene::viewport::{
    RenderFrameExtract, RenderSceneSnapshot, SceneViewportChromeSettings,
    SceneViewportWorkspaceSessionSnapshot,
};
use crate::ui::host::editor_activity_log::activity_log_console_output_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, EditorDataSnapshot, TransactionHistorySnapshot,
};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};

fn manager_view_instance_id(
    view_id: &crate::core::editor_event::ViewInstanceId,
) -> crate::ui::workbench::view::ViewInstanceId {
    crate::ui::workbench::view::ViewInstanceId::new(view_id.0.clone())
}

fn core_view_instance_id(
    view_id: &crate::ui::workbench::view::ViewInstanceId,
) -> crate::core::editor_event::ViewInstanceId {
    crate::core::editor_event::ViewInstanceId::new(view_id.0.clone())
}

impl EditorHostEventController {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        let play_inspector = match self.active_hierarchy_world_domain() {
            WorldDomain::Play(_) => Some(self.play_inspector_snapshot()),
            WorldDomain::Edit => None,
        };
        let mut inner = self.shell().lock();
        let inspector_customizations = Self::active_inspector_customizations_for_shell(&inner);
        let field_editors = Self::active_field_editors_for_shell(&inner);
        let mut snapshot = inner
            .state
            .snapshot_with_inspector_customizations(&inspector_customizations, &field_editors);
        if let Err(error) = Self::project_asset_type_registry_for_shell(&mut inner, &mut snapshot) {
            Self::present_asset_type_registry_projection_error(&mut snapshot, error);
        }
        snapshot.console_output = activity_log_console_output_for_shell(&mut inner);
        if let Some(play_inspector) = play_inspector {
            snapshot.inspector = play_inspector;
        }
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

    pub fn active_scene_transaction_history_snapshot(
        &self,
    ) -> Result<Option<TransactionHistorySnapshot>, EditCommandError> {
        self.shell()
            .lock()
            .state
            .active_scene_transaction_history_snapshot()
    }

    pub fn scene_viewport_settings(&self) -> SceneViewportChromeSettings {
        self.shell().lock().state.scene_viewport_settings()
    }

    pub(crate) fn scene_viewport_settings_for_view(
        &self,
        view_id: &crate::core::editor_event::ViewInstanceId,
    ) -> Option<SceneViewportChromeSettings> {
        self.shell()
            .lock()
            .state
            .viewport_controller
            .chrome_settings_for_view(view_id)
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

    /// Returns an owned snapshot of the authoritative editor scene after pending bindings apply.
    pub(crate) fn project_scene_snapshot(
        &self,
    ) -> Result<
        Option<zircon_runtime::scene::Scene>,
        crate::core::editing::authoring_world::AuthoringWorldAccessError,
    > {
        self.shell().lock().state.project_scene()
    }

    pub fn render_frame_extract(&self) -> Option<RenderFrameExtract> {
        self.shell().lock().state.render_frame_extract()
    }

    pub(crate) fn render_frame_submission(&self) -> Option<EditorRenderFrameSubmission> {
        self.shell().lock().state.render_frame_submission()
    }

    pub(crate) fn render_frame_submission_for_view(
        &self,
        view_id: &crate::core::editor_event::ViewInstanceId,
        size: zircon_runtime_interface::math::UVec2,
        runtime_viewport: zircon_runtime_interface::ZrRuntimeViewportHandle,
    ) -> Option<EditorRenderFrameSubmission> {
        self.shell()
            .lock()
            .state
            .render_frame_submission_for_view(view_id, size, runtime_viewport)
    }

    pub(crate) fn route_scene_viewport_pointer(
        &self,
        view_id: crate::core::editor_event::ViewInstanceId,
        event_kind: zircon_runtime_interface::ui::surface::UiPointerEventKind,
    ) -> bool {
        use zircon_runtime_interface::ui::surface::UiPointerEventKind;

        if matches!(
            event_kind,
            UiPointerEventKind::Down | UiPointerEventKind::Scroll
        ) {
            self.focus_scene_viewport(view_id)
        } else {
            // Hover and terminal events select a live routing session without changing saved focus.
            let mut shell = self.shell().lock();
            let manager_view_id = manager_view_instance_id(&view_id);
            if !shell
                .manager
                .view_instance_ids_for_descriptor_key("editor.scene")
                .contains(&manager_view_id)
            {
                return false;
            }
            shell.state.focus_scene_viewport(view_id);
            true
        }
    }

    pub(crate) fn focus_scene_viewport(
        &self,
        view_id: crate::core::editor_event::ViewInstanceId,
    ) -> bool {
        let mut shell = self.shell().lock();
        let manager_view_id = manager_view_instance_id(&view_id);
        if !shell
            .manager
            .view_instance_ids_for_descriptor_key("editor.scene")
            .contains(&manager_view_id)
        {
            return false;
        }
        if shell.manager.focus_view(&manager_view_id).is_err() {
            return false;
        }
        if shell.manager.current_focused_view().as_ref() != Some(&manager_view_id) {
            return false;
        }
        shell.state.focus_scene_viewport(view_id);
        true
    }

    pub(crate) fn retain_scene_viewports(
        &self,
        retained: &std::collections::BTreeSet<crate::core::editor_event::ViewInstanceId>,
    ) {
        self.shell().lock().state.retain_scene_viewports(retained);
    }

    /// Seeds every manager-live Scene leaf at the owning Workbench commit boundary. Retained
    /// presentation may later retire sessions, but opening or restoring a leaf must establish its
    /// session before a callback can target that committed view.
    pub(crate) fn ensure_live_scene_viewport_sessions(&self) {
        let mut shell = self.shell().lock();
        let live_views = shell
            .manager
            .view_instance_ids_for_descriptor_key("editor.scene");
        for view_id in live_views {
            shell
                .state
                .ensure_scene_viewport_session(core_view_instance_id(&view_id));
        }
    }

    pub(crate) fn scene_viewport_workspace_sessions(
        &self,
    ) -> BTreeMap<crate::core::editor_event::ViewInstanceId, SceneViewportWorkspaceSessionSnapshot>
    {
        let shell = self.shell().lock();
        let live_views = shell
            .manager
            .view_instance_ids_for_descriptor_key("editor.scene")
            .into_iter()
            .map(|view_id| core_view_instance_id(&view_id))
            .collect::<std::collections::BTreeSet<_>>();
        shell
            .state
            .viewport_controller
            .snapshot_workspace_sessions(&live_views)
    }

    pub(crate) fn restore_scene_viewport_workspace_sessions(
        &self,
        snapshots: &BTreeMap<
            crate::core::editor_event::ViewInstanceId,
            SceneViewportWorkspaceSessionSnapshot,
        >,
        focused_view: Option<&crate::core::editor_event::ViewInstanceId>,
    ) {
        let mut shell = self.shell().lock();
        let live_views = shell
            .manager
            .view_instance_ids_for_descriptor_key("editor.scene")
            .into_iter()
            .map(|view_id| core_view_instance_id(&view_id))
            .collect::<std::collections::BTreeSet<_>>();
        shell.state.viewport_controller.restore_workspace_sessions(
            &live_views,
            snapshots,
            focused_view,
        );
    }

    pub fn viewport_state(&self) -> crate::scene::viewport::ViewportState {
        self.shell().lock().state.viewport_state()
    }
}

#[cfg(test)]
#[path = "tests/snapshot_scene_viewport_route_tests.rs"]
mod scene_viewport_route_tests;
