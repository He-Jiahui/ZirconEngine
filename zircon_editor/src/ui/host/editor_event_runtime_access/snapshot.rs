use crate::scene::viewport::{RenderFrameExtract, RenderSceneSnapshot};
use crate::ui::host::editor_activity_log::activity_log_console_output;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, EditorDataSnapshot};
use crate::ui::workbench::state::EditorRenderFrameSubmission;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};

impl EditorHostEventController {
    pub fn editor_snapshot(&self) -> EditorDataSnapshot {
        let mut inner = self.shell().lock();
        let inspector_customizations = Self::active_inspector_customizations_for_shell(&inner);
        let field_editors = Self::active_field_editors_for_shell(&inner);
        let mut snapshot = inner
            .state
            .snapshot_with_inspector_customizations(&inspector_customizations, &field_editors);
        Self::project_asset_type_registry_for_shell(&mut inner, &mut snapshot);
        snapshot.console_output = activity_log_console_output(
            inner.manager.context().logs(),
            inner.console_message_filter,
            inner.console_source_filter,
        );
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

    /// Returns an owned snapshot of the authoritative editor scene after pending bindings apply.
    pub(crate) fn project_scene_snapshot(&self) -> Option<zircon_runtime::scene::Scene> {
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
