use crate::core::commands::CommandEvalCtx;
use crate::core::editing::interactive_transform::PivotMode;
use crate::core::settings::{SettingValue, SettingsKey, SettingsScope};
use crate::scene::modes::{SceneModeActivation, SceneModeCtx};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::{
    GizmoAxis, SceneViewportChromeSettings, SceneViewportSettings, SceneViewportSnapSteps,
    ViewportState,
};
use zircon_runtime_interface::math::Vec3;

use super::{SceneViewportController, SceneViewportControllerError};

impl SceneViewportController {
    pub(crate) fn viewport(&self) -> &ViewportState {
        &self.state.viewport
    }

    pub(crate) fn hovered_axis(&self) -> Option<GizmoAxis> {
        self.state.hover.hovered_axis
    }

    pub(crate) fn settings(&self) -> &SceneViewportSettings {
        &self.state.settings
    }

    pub(crate) fn settings_mut(&mut self) -> &mut SceneViewportSettings {
        &mut self.state.settings
    }

    pub(crate) fn chrome_settings(&self) -> SceneViewportChromeSettings {
        SceneViewportChromeSettings::new(
            &self.state.settings,
            self.snap_steps(),
            self.active_scene_mode(),
            self.interactive_transform_pivot_mode(),
        )
    }

    pub(crate) fn active_scene_mode(&self) -> SceneModeActivation {
        self.state.scene_modes.active_activation()
    }

    fn base_scene_mode(&self) -> SceneModeActivation {
        self.state.scene_modes.base_activation().clone()
    }

    pub(in crate::scene::viewport::controller) fn base_transform_handle(
        &self,
    ) -> Option<crate::scene::viewport::TransformHandleKind> {
        self.state.scene_modes.base_transform_handle()
    }

    pub(crate) fn project_command_eval_ctx(&self, context: CommandEvalCtx) -> CommandEvalCtx {
        self.state
            .scene_modes
            .project_command_eval_ctx(context, &self.state.selection)
    }

    pub(crate) fn activate_scene_mode(
        &mut self,
        activation: SceneModeActivation,
    ) -> Result<bool, SceneViewportControllerError> {
        activation.validate()?;
        if self.base_scene_mode() == activation {
            return Ok(false);
        }
        let acquired_now = self.scene_tool_lease.is_none();
        self.ensure_scene_tool_lease()?;

        let mode_id = activation.mode_id();
        let (mode, contribution_ticket) = match self
            .state
            .scene_mode_registry
            .create_with_contribution(&mode_id)
        {
            Ok(created) => created,
            Err(error) => {
                if acquired_now {
                    self.release_scene_tool_lease();
                }
                return Err(error.into());
            }
        };
        let replacement = {
            let state = &mut self.state;
            let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state
                .scene_modes
                .replace_base_with_contribution(
                    activation,
                    mode,
                    contribution_ticket,
                    &mut mode_ctx,
                )
                .map(|(retired, _)| drop(retired))
        };
        if let Err(error) = replacement {
            if acquired_now {
                self.release_scene_tool_lease();
            }
            return Err(error);
        }
        if !self.state.scene_modes.requires_exclusive_tool() {
            self.release_scene_tool_lease();
        }
        self.interaction_extract.invalidate();
        Ok(true)
    }

    pub(crate) fn selection(&self) -> &SelectionModel {
        &self.state.selection
    }

    pub(crate) fn selection_mut(&mut self) -> &mut SelectionModel {
        &mut self.state.selection
    }

    pub(crate) const fn interactive_transform_pivot_mode(&self) -> PivotMode {
        self.state.pivot_mode
    }

    pub(crate) fn set_interactive_transform_pivot_mode(&mut self, mode: PivotMode) -> bool {
        let changed = self.state.pivot_mode != mode;
        if changed {
            self.state.pivot_mode = mode;
            self.interaction_extract.invalidate();
        }
        changed
    }

    pub(crate) fn set_orbit_target(&mut self, target: Vec3) {
        self.state.orbit_target = target;
        self.state.orbit_controller.set_target(target);
    }

    pub(crate) fn orbit_target(&self) -> Vec3 {
        self.state.orbit_target
    }

    pub(crate) fn is_handle_drag_active(&self) -> bool {
        matches!(
            self.state.drag,
            Some(super::viewport_drag_session::ViewportDragSession::Handle { .. })
        )
    }

    pub(in crate::scene::viewport::controller) fn set_project_snap_step(
        &mut self,
        key: &str,
        step: f32,
    ) -> Result<bool, SceneViewportControllerError> {
        let key = SettingsKey::parse(key).expect("the built-in viewport snap key is valid");
        let receipt = self.settings_mutations.set(
            SettingsScope::Project,
            &key,
            SettingValue::Float(f64::from(step)),
        )?;
        Ok(receipt.changed())
    }

    pub(in crate::scene::viewport::controller) fn snap_steps(&self) -> SceneViewportSnapSteps {
        let snap = self
            .settings_mutations
            .authority()
            .snapshot()
            .viewport_snap();
        SceneViewportSnapSteps {
            translate_step: snap.translate_step() as f32,
            rotate_step_deg: snap.rotate_step_degrees() as f32,
            scale_step: snap.scale_step() as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use zircon_runtime_interface::math::UVec2;

    use crate::core::editor_message::SceneModeId;
    use crate::core::settings::{
        SettingValue, SettingsKey, SettingsMutationCoordinator, SettingsScope,
        VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeActivationError, SceneModeCtx,
        SceneModeRegistry, SceneModeRegistryError, ViewportOverlayBuilder,
    };
    use crate::scene::viewport::{TransformHandleKind, ViewportInput};

    use super::{SceneViewportController, SceneViewportControllerError};

    struct OverlayMode {
        id: SceneModeId,
    }

    impl OverlayMode {
        fn new(id: &str) -> Self {
            Self {
                id: SceneModeId::new(id),
            }
        }
    }

    impl EditorSceneMode for OverlayMode {
        fn id(&self) -> &SceneModeId {
            &self.id
        }

        fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

        fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

        fn handle_input(
            &mut self,
            _input: &ViewportInput,
            _ctx: &mut SceneModeCtx<'_>,
        ) -> InputOutcome {
            InputOutcome::PassThrough
        }

        fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
    }

    #[test]
    fn snap_projection_reads_the_shared_authority_typed_slot() {
        let coordinator = Arc::new(SettingsMutationCoordinator::in_memory_with_defaults());
        let authority = Arc::clone(coordinator.authority());
        let controller =
            SceneViewportController::with_settings_coordinator(UVec2::new(1280, 720), coordinator);
        let translate_key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        authority
            .set(
                SettingsScope::Project,
                &translate_key,
                SettingValue::Float(3.5),
            )
            .unwrap();

        assert_eq!(controller.snap_steps().translate_step, 3.5);
    }

    #[test]
    fn transform_handle_activation_replaces_the_stack_base_without_settings_state() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        assert!(controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move,))
            .unwrap());
        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Transform(TransformHandleKind::Move)
        );
        assert_eq!(
            controller.base_transform_handle(),
            Some(TransformHandleKind::Move)
        );
        assert!(!controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move,))
            .unwrap());
        assert!(controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Rotate))
            .unwrap());
        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Transform(TransformHandleKind::Rotate)
        );
        assert_eq!(
            controller.base_transform_handle(),
            Some(TransformHandleKind::Rotate)
        );
    }

    #[test]
    fn failed_scene_mode_activation_rolls_back_transform_handle_configuration() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        controller
            .activate_scene_mode(SceneModeActivation::Select)
            .unwrap();
        controller.state.scene_mode_registry = SceneModeRegistry::default();

        let error = controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Rotate))
            .unwrap_err();

        assert!(matches!(
            error,
            SceneViewportControllerError::SceneModeRegistry(
                SceneModeRegistryError::UnknownMode { mode_id }
            ) if mode_id == SceneModeId::new("scene.transform")
        ));
        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
        assert_eq!(controller.base_transform_handle(), None);
    }

    #[test]
    fn custom_scene_mode_activation_rejects_reserved_builtin_ids() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let error = controller
            .activate_scene_mode(SceneModeActivation::Custom(SceneModeId::new(
                "scene.select",
            )))
            .unwrap_err();

        assert!(matches!(
            error,
            SceneViewportControllerError::SceneModeActivation(
                SceneModeActivationError::ReservedBuiltInId { mode_id }
            ) if mode_id == SceneModeId::new("scene.select")
        ));
        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
    }

    #[test]
    fn active_scene_mode_tracks_the_overlay_stack_top() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        {
            let state = &mut controller.state;
            let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state
                .scene_modes
                .push_overlay(
                    SceneModeActivation::Custom(SceneModeId::new("test.overlay")),
                    Box::new(OverlayMode::new("test.overlay")),
                    &mut mode_ctx,
                )
                .unwrap();
        }

        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Custom(SceneModeId::new("test.overlay"))
        );
        assert_eq!(controller.base_transform_handle(), None);
    }
}
