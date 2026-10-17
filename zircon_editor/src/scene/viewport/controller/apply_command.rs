use crate::scene::viewport::{GridMode, TransformHandleKind, ViewportFeedback};
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::Scene;

use crate::core::settings::{
    SettingValue, SettingsKey, SettingsScope, VIEWPORT_ROTATE_SNAP_ENABLED_KEY,
    VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_SNAP_ENABLED_KEY, VIEWPORT_SCALE_STEP_KEY,
    VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
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
                // The legacy combined command keeps its explicit all-snap
                // meaning. The authored UE Show menu uses visibility only.
                let enabled = *mode == GridMode::VisibleAndSnap;
                for kind in [
                    TransformHandleKind::Move,
                    TransformHandleKind::Rotate,
                    TransformHandleKind::Scale,
                ] {
                    feedback.settings_changed |= self.set_snap_enabled(kind, enabled)?;
                }
                feedback.settings_changed |=
                    replace_if_changed(&mut self.state.settings.grid_mode, *mode);
            }
            ViewportCommand::SetGridVisibility(visible) => {
                let mode = if *visible {
                    GridMode::VisibleNoSnap
                } else {
                    GridMode::Hidden
                };
                feedback.settings_changed =
                    replace_if_changed(&mut self.state.settings.grid_mode, mode);
            }
            ViewportCommand::ToggleSnap(kind) => {
                let resolved = self.snap_steps();
                let current = match kind {
                    TransformHandleKind::Move => resolved.translate_enabled,
                    TransformHandleKind::Rotate => resolved.rotate_enabled,
                    TransformHandleKind::Scale => resolved.scale_enabled,
                };
                feedback.settings_changed = self.set_snap_enabled(*kind, !current)?;
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

impl SceneViewportController {
    fn set_snap_enabled(
        &mut self,
        kind: TransformHandleKind,
        enabled: bool,
    ) -> Result<bool, SceneViewportControllerError> {
        let key = match kind {
            TransformHandleKind::Move => VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY,
            TransformHandleKind::Rotate => VIEWPORT_ROTATE_SNAP_ENABLED_KEY,
            TransformHandleKind::Scale => VIEWPORT_SCALE_SNAP_ENABLED_KEY,
        };
        let key = SettingsKey::parse(key).expect("the built-in snap-enable key is valid");
        let receipt = self.settings_mutations.set(
            SettingsScope::Project,
            &key,
            SettingValue::Bool(enabled),
        )?;
        Ok(receipt.changed())
    }
}

#[cfg(test)]
#[path = "tests/unreal_snap_command.rs"]
mod unreal_snap_command;

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
