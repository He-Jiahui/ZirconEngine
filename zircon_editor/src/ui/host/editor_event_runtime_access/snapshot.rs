use crate::core::editing::engine::EditCommandError;
use crate::core::play::WorldDomain;
use crate::scene::viewport::{
    RenderFrameExtract, RenderSceneSnapshot, SceneViewportChromeSettings,
};
use crate::ui::host::editor_activity_log::activity_log_console_output_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, EditorDataSnapshot, TransactionHistorySnapshot,
};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};

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

    pub fn viewport_state(&self) -> crate::scene::viewport::ViewportState {
        self.shell().lock().state.viewport_state()
    }
}
