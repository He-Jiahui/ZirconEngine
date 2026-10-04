use super::*;
use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{EditorSceneMode, SceneModeActivation, ViewportOverlayBuilder};
use crate::scene::viewport::GizmoAxis;
use zircon_runtime_interface::{
    math::UVec2,
    ui::{event_ui::UiNodeId, tree::UiTreeError},
};

struct ConsumingPointerMode {
    id: SceneModeId,
}

impl ConsumingPointerMode {
    fn new() -> Self {
        Self {
            id: SceneModeId::new("test.consume-pointer"),
        }
    }
}

impl EditorSceneMode for ConsumingPointerMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        let outcome = matches!(input, ViewportInput::LeftPressed { .. })
            .then_some(InputOutcome::Consumed)
            .unwrap_or(InputOutcome::PassThrough);
        if outcome == InputOutcome::Consumed {
            ctx.invalidate_overlay();
        }
        outcome
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

#[test]
fn consumed_scene_mode_input_does_not_start_builtin_primary_navigation() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    {
        let state = &mut controller.state;
        let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state
            .scene_modes
            .push_overlay(
                SceneModeActivation::Custom(SceneModeId::new("test.consume-pointer")),
                Box::new(ConsumingPointerMode::new()),
                &mut mode_ctx,
            )
            .unwrap();
    }

    let feedback = controller
        .handle_input(
            &mut Scene::new(),
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        )
        .unwrap();

    assert!(feedback.transformed_node.is_none());
    assert!(controller.state.drag.is_none());
}

#[test]
fn scene_mode_input_overlay_invalidation_rebuilds_the_shared_extract() {
    use std::sync::Arc;

    use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};

    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    {
        let state = &mut controller.state;
        let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state
            .scene_modes
            .push_overlay(
                SceneModeActivation::Custom(SceneModeId::new("test.consume-pointer")),
                Box::new(ConsumingPointerMode::new()),
                &mut mode_ctx,
            )
            .unwrap();
    }
    let scene = Scene::new();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = UVec2::new(1280, 720);
    controller.build_render_snapshot(&scene);
    let before = controller
        .interaction_extract
        .resolve_for_pointer(&scene, None, &settings, &camera, viewport);
    let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(before) = before
    else {
        panic!("the render path must publish the initial interaction extract");
    };

    let feedback = controller
        .handle_input(
            &mut Scene::new(),
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        )
        .unwrap();
    assert!(
        feedback.interaction_extract_stale,
        "overlay invalidation must schedule publication of a replacement render product"
    );

    let after = controller
        .interaction_extract
        .resolve_for_pointer(&scene, None, &settings, &camera, viewport);
    assert!(matches!(
        after,
        crate::scene::viewport::ViewportInteractionExtractPointerResolution::Stale
    ));
    controller.build_render_snapshot(&scene);
    let after = controller
        .interaction_extract
        .resolve_for_pointer(&scene, None, &settings, &camera, viewport);
    let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(after) = after
    else {
        panic!("the render path must rebuild the invalidated interaction extract");
    };
    assert!(!Arc::ptr_eq(&before, &after));
}

#[test]
fn stale_pointer_product_rejects_press_until_render_publishes_a_current_extract() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    let mut scene = Scene::new();

    let stale = controller
        .handle_input(
            &mut scene,
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        )
        .unwrap();
    assert!(stale.interaction_extract_stale);
    assert!(controller.state.drag.is_none());

    let preparing = controller
        .handle_input(
            &mut scene,
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        )
        .unwrap();
    assert!(!preparing.interaction_extract_stale);
    assert!(controller.state.drag.is_none());

    controller.build_render_snapshot(&scene);
    let current = controller
        .handle_input(&mut scene, ViewportInput::PointerMoved(Vec2::ZERO))
        .unwrap();
    assert!(!current.interaction_extract_stale);
}

#[test]
fn pointer_route_error_clears_stale_hover_without_erasing_the_error_kind() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.state.hover.hovered_axis = Some(GizmoAxis::Z);
    controller.state.hover.hovered_entity = Some(91);
    let expected = UiTreeError::MissingNode(UiNodeId::new(901));

    let error = controller.clear_hover_after_pointer_route_error(expected.clone());

    assert_eq!(error, expected);
    assert!(controller.state.hover.hovered_axis.is_none());
    assert!(controller.state.hover.hovered_entity.is_none());
}

#[test]
fn transform_mode_input_only_publishes_transaction_preview_requests() {
    let source = include_str!("../scene_viewport_controller_handle_input.rs");
    let production_source = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production);

    assert!(production_source.contains("feedback.transform_request"));
    assert!(!production_source.contains("scene.update_transform("));
}

#[test]
fn editor_camera_navigation_entry_does_not_dispatch_primary_selection_to_scene_modes() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    {
        let state = &mut controller.state;
        let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state
            .scene_modes
            .push_overlay(
                SceneModeActivation::Custom(SceneModeId::new("test.consume-pointer")),
                Box::new(ConsumingPointerMode::new()),
                &mut mode_ctx,
            )
            .unwrap();
    }

    let feedback = controller
        .handle_editor_camera_input(
            &Scene::new(),
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        )
        .unwrap();

    assert!(!feedback.camera_updated);
    assert!(feedback.hovered_axis.is_none());
    assert!(feedback.transformed_node.is_none());
    assert!(feedback.transform_request.is_none());
    assert!(controller.state.drag.is_none());
}
