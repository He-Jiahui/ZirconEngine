use crate::ViewportFeedback;
use crate::ui::ViewportCommand;
use zircon_scene::Scene;

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn apply_command(
        &mut self,
        scene: Option<&Scene>,
        command: &ViewportCommand,
    ) -> ViewportFeedback {
        let mut feedback = ViewportFeedback::default();

        match command {
            ViewportCommand::SetTool(tool) => self.state.settings.tool = *tool,
            ViewportCommand::SetTransformSpace(space) => {
                self.state.settings.transform_space = *space
            }
            ViewportCommand::SetProjectionMode(mode) => self.set_projection_mode(*mode),
            ViewportCommand::AlignView(orientation) => self.align_view(*orientation),
            ViewportCommand::SetDisplayMode(mode) => self.state.settings.display_mode = *mode,
            ViewportCommand::SetGridMode(mode) => self.state.settings.grid_mode = *mode,
            ViewportCommand::SetTranslateSnap(step) => {
                self.state.settings.translate_step = step.max(0.0001)
            }
            ViewportCommand::SetRotateSnapDegrees(step) => {
                self.state.settings.rotate_step_deg = step.max(0.0001)
            }
            ViewportCommand::SetScaleSnap(step) => {
                self.state.settings.scale_step = step.max(0.0001)
            }
            ViewportCommand::SetPreviewLighting(enabled) => {
                self.state.settings.preview_lighting = *enabled
            }
            ViewportCommand::SetPreviewSkybox(enabled) => {
                self.state.settings.preview_skybox = *enabled
            }
            ViewportCommand::SetGizmosEnabled(enabled) => {
                self.state.settings.gizmos_enabled = *enabled
            }
            ViewportCommand::FrameSelection => {
                if let Some(scene) = scene {
                    feedback.camera_updated = self.frame_selection(scene);
                }
            }
            _ => {}
        }

        feedback
    }
}
