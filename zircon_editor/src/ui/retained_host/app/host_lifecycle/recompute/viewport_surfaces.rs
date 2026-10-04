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
        let command_context = self.runtime.command_eval_ctx_for_source(
            &crate::core::editor_operation::EditorOperationSource::UiBinding,
        );
        let (enter, exit) = {
            let commands = self.runtime.context().commands().lock();
            (
                commands
                    .command("runtime.play_mode.enter")
                    .is_some_and(|command| command.is_enabled(&command_context)),
                commands
                    .command("runtime.play_mode.exit")
                    .is_some_and(|command| command.is_enabled(&command_context)),
            )
        };
        self.viewport_toolbar_bridge.set_play_admission(
            enter,
            exit,
            self.runtime.play_sessions().mode() == crate::core::play::PlayModeKind::Playing,
        );
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
