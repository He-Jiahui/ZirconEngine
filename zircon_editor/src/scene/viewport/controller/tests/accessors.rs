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
