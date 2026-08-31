use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::Scene;

use crate::core::settings::{
    VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};

use super::{SceneViewportController, SceneViewportControllerError};

impl SceneViewportController {
    pub(crate) fn apply_command(
        &mut self,
        scene: Option<&Scene>,
        command: &ViewportCommand,
    ) -> Result<ViewportFeedback, SceneViewportControllerError> {
        let mut feedback = ViewportFeedback::default();

        match command {
            ViewportCommand::ActivateSceneMode(mode) => {
                self.activate_scene_mode(mode.clone())?;
            }
            ViewportCommand::SetTransformSpace(space) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.transform_space, *space);
            }
            ViewportCommand::SetPivotMode(mode) => {
                feedback.settings_changed = self.set_interactive_transform_pivot_mode(*mode);
                feedback.interaction_extract_stale = feedback.settings_changed;
            }
            ViewportCommand::SetProjectionMode(mode) => {
                feedback.settings_changed = self.state.settings.projection_mode != *mode;
                if feedback.settings_changed {
                    self.set_projection_mode(*mode);
                }
            }
            ViewportCommand::AlignView(orientation) => {
                feedback.settings_changed = self.state.settings.view_orientation != *orientation;
                self.align_view(*orientation);
            }
            ViewportCommand::SetDisplayMode(mode) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.display_mode, *mode);
            }
            ViewportCommand::SetGridMode(mode) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.grid_mode, *mode);
            }
            ViewportCommand::SetTranslateSnap(step) => {
                feedback.settings_changed = self.set_project_snap_step(
                    VIEWPORT_TRANSLATE_STEP_KEY,
                    validated_snap_step(*step)?,
                )?;
            }
            ViewportCommand::SetRotateSnapDegrees(step) => {
                feedback.settings_changed = self.set_project_snap_step(
                    VIEWPORT_ROTATE_STEP_DEGREES_KEY,
                    validated_snap_step(*step)?,
                )?;
            }
            ViewportCommand::SetScaleSnap(step) => {
                feedback.settings_changed = self
                    .set_project_snap_step(VIEWPORT_SCALE_STEP_KEY, validated_snap_step(*step)?)?;
            }
            ViewportCommand::SetPreviewLighting(enabled) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.preview_lighting, *enabled);
            }
            ViewportCommand::SetPreviewSkybox(enabled) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.preview_skybox, *enabled);
            }
            ViewportCommand::SetGizmosEnabled(enabled) => {
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.gizmos_enabled, *enabled);
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

fn replace_if_changed<T>(slot: &mut T, value: T) -> bool
where
    T: Copy + PartialEq,
{
    if *slot == value {
        return false;
    }
    *slot = value;
    true
}

fn validated_snap_step(step: f32) -> Result<f32, SceneViewportControllerError> {
    if !step.is_finite() {
        return Err(SceneViewportControllerError::InvalidSnapStep { value: step });
    }
    Ok(step.max(0.0001))
}
