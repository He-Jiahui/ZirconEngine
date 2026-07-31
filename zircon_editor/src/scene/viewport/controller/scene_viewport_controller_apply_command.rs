use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::Scene;

use crate::core::settings::{
    VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn apply_command(
        &mut self,
        scene: Option<&Scene>,
        command: &ViewportCommand,
    ) -> Result<ViewportFeedback, String> {
        let mut feedback = ViewportFeedback::default();

        match command {
            ViewportCommand::ActivateSceneMode(mode) => {
                self.activate_scene_mode(mode.clone())?;
            }
            ViewportCommand::SetTransformSpace(space) => {
                self.state.settings.transform_space = *space
            }
            ViewportCommand::SetProjectionMode(mode) => self.set_projection_mode(*mode),
            ViewportCommand::AlignView(orientation) => self.align_view(*orientation),
            ViewportCommand::SetDisplayMode(mode) => self.state.settings.display_mode = *mode,
            ViewportCommand::SetGridMode(mode) => self.state.settings.grid_mode = *mode,
            ViewportCommand::SetTranslateSnap(step) => self
                .set_project_snap_step(VIEWPORT_TRANSLATE_STEP_KEY, validated_snap_step(*step)?)?,
            ViewportCommand::SetRotateSnapDegrees(step) => self.set_project_snap_step(
                VIEWPORT_ROTATE_STEP_DEGREES_KEY,
                validated_snap_step(*step)?,
            )?,
            ViewportCommand::SetScaleSnap(step) => {
                self.set_project_snap_step(VIEWPORT_SCALE_STEP_KEY, validated_snap_step(*step)?)?
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
            ViewportCommand::ToggleOverlayProvider { provider_id } => {
                self.toggle_viewport_overlay_provider(provider_id)?;
            }
            ViewportCommand::FrameSelection => {
                if let Some(scene) = scene {
                    feedback.camera_updated = self.frame_selection(scene);
                }
            }
            _ => {}
        }

        Ok(feedback)
    }
}

fn validated_snap_step(step: f32) -> Result<f32, String> {
    if !step.is_finite() {
        return Err("viewport snap step must be finite".to_string());
    }
    Ok(step.max(0.0001))
}
