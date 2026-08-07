use super::super::super::RetainedEditorHost;
use crate::ui::retained_host::app::viewport_toolbar_projection::attach_viewport_toolbar_surface_frames_to_ui;
use crate::ui::retained_host::callback_dispatch;

impl RetainedEditorHost {
    pub(super) fn sync_recompute_viewport_surfaces(
        &mut self,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
    ) {
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_viewport_surfaces");
        let document_viewport_toolbar_width = componentized_workbench_layout_frames
            .viewport_toolbar_frame
            .map(|frame| frame.width);
        attach_viewport_toolbar_surface_frames_to_ui(
            &self.ui,
            &mut self.viewport_toolbar_bridge,
            document_viewport_toolbar_width,
        );
        let generation = self.ui.get_host_presentation_generation();
        let world_space_ui_surfaces =
            crate::ui::retained_host::build_world_space_ui_surface_submissions_from_host_scene(
                &generation.structure().host_scene_data,
            );
        self.viewport
            .submit_world_space_ui_surfaces(world_space_ui_surfaces);
    }
}
