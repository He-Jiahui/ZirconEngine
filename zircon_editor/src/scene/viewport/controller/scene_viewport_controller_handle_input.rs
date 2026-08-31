use crate::scene::modes::{InputOutcome, SceneModeCtx, SceneModeInputEffect};
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::ViewportInput;
use crate::scene::viewport::{ViewportFeedback, ViewportTransformRequest};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::{math::Vec2, ui::tree::UiTreeError};

use crate::scene::viewport::pointer::ViewportPointerRoute;

use super::{
    constants::PRIMARY_NAV_THRESHOLD,
    scene_viewport_controller_pointer_route::{route_owner, PointerBridgeProductState},
    viewport_drag_session::ViewportDragSession,
    SceneViewportController,
};

impl SceneViewportController {
    pub(crate) fn handle_input(
        &mut self,
        scene: &mut Scene,
        input: ViewportInput,
    ) -> Result<ViewportFeedback, UiTreeError> {
        let mut feedback = ViewportFeedback::default();

        // A resize maintains the controller's viewport invariant even when a mode owns the
        // event. All other builtin interaction runs only after the mode stack passes through.
        if let ViewportInput::Resized(size) = &input {
            self.apply_viewport_size(*size);
        }
        let (outcome, effect, overlay_invalidated) = self.dispatch_scene_mode_input(&input);
        if overlay_invalidated {
            self.interaction_extract.invalidate();
            feedback.interaction_extract_stale = true;
        }
        if outcome == InputOutcome::Consumed {
            if let Some(effect) = effect {
                self.apply_scene_mode_input_effect(scene, effect, &mut feedback)?;
            }
            return Ok(feedback);
        }

        self.apply_editor_camera_input(scene, input, &mut feedback, true);

        Ok(feedback)
    }

    /// Handles only the editor camera controls that remain valid while SIE displays a Play world.
    ///
    /// This entry intentionally bypasses Scene Mode, authoring hit tests, selection drags, and
    /// transform handles. The authoring scene is used only to seed an uninitialized editor camera.
    pub(crate) fn handle_editor_camera_input(
        &mut self,
        scene: &Scene,
        input: ViewportInput,
    ) -> Result<ViewportFeedback, UiTreeError> {
        let mut feedback = ViewportFeedback::default();
        self.apply_editor_camera_input(scene, input, &mut feedback, false);
        Ok(feedback)
    }

    fn apply_editor_camera_input(
        &mut self,
        scene: &Scene,
        input: ViewportInput,
        feedback: &mut ViewportFeedback,
        resize_already_applied: bool,
    ) {
        match input {
            ViewportInput::Resized(size) => {
                if !resize_already_applied {
                    self.apply_viewport_size(size);
                }
            }
            ViewportInput::PointerMoved(position) => match self.state.drag.take() {
                Some(ViewportDragSession::Orbit { last }) => {
                    feedback.camera_updated = self.apply_orbit(last, position);
                    self.state.drag = Some(ViewportDragSession::Orbit { last: position });
                }
                Some(ViewportDragSession::Pan { last }) => {
                    feedback.camera_updated = self.apply_pan(last, position);
                    self.state.drag = Some(ViewportDragSession::Pan { last: position });
                }
                drag => self.state.drag = drag,
            },
            ViewportInput::LeftPressed { .. } | ViewportInput::LeftReleased => {}
            ViewportInput::RightPressed(position) => {
                if self.state.drag.is_some() {
                    return;
                }
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                self.state.drag = Some(ViewportDragSession::Orbit { last: position });
            }
            ViewportInput::RightReleased => {
                if matches!(self.state.drag, Some(ViewportDragSession::Orbit { .. })) {
                    self.state.drag = None;
                }
            }
            ViewportInput::MiddlePressed(position) => {
                if self.state.drag.is_some() {
                    return;
                }
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                self.state.drag = Some(ViewportDragSession::Pan { last: position });
            }
            ViewportInput::MiddleReleased => {
                if matches!(self.state.drag, Some(ViewportDragSession::Pan { .. })) {
                    self.state.drag = None;
                }
            }
            ViewportInput::Scrolled(delta) => {
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                feedback.camera_updated = self.apply_zoom(delta);
            }
        }
    }

    fn dispatch_scene_mode_input(
        &mut self,
        input: &ViewportInput,
    ) -> (InputOutcome, Option<SceneModeInputEffect>, bool) {
        let state = &mut self.state;
        let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        let outcome = state.scene_modes.handle_input(input, &mut mode_ctx);
        (
            outcome,
            mode_ctx.take_input_effect(),
            mode_ctx.take_overlay_invalidation(),
        )
    }

    fn apply_scene_mode_input_effect(
        &mut self,
        scene: &mut Scene,
        effect: SceneModeInputEffect,
        feedback: &mut ViewportFeedback,
    ) -> Result<(), UiTreeError> {
        match effect {
            SceneModeInputEffect::PointerMoved(position) => {
                self.handle_pointer_moved(scene, position, feedback)?;
            }
            SceneModeInputEffect::SelectionPrimaryPressed {
                position,
                selection_mutation,
            } => {
                self.handle_selection_primary_pressed(
                    scene,
                    position,
                    selection_mutation,
                    feedback,
                )?;
            }
            SceneModeInputEffect::TransformPrimaryPressed {
                position,
                selection_mutation,
            } => {
                self.handle_transform_primary_pressed(
                    scene,
                    position,
                    selection_mutation,
                    feedback,
                )?;
            }
            SceneModeInputEffect::PrimaryReleased => self.handle_left_released(scene),
        }
        Ok(())
    }

    fn handle_pointer_moved(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        feedback: &mut ViewportFeedback,
    ) -> Result<(), UiTreeError> {
        match self.state.drag.take() {
            Some(ViewportDragSession::Orbit { last }) => {
                feedback.camera_updated = self.apply_orbit(last, position);
                self.state.drag = Some(ViewportDragSession::Orbit { last: position });
            }
            Some(ViewportDragSession::Pan { last }) => {
                feedback.camera_updated = self.apply_pan(last, position);
                self.state.drag = Some(ViewportDragSession::Pan { last: position });
            }
            Some(ViewportDragSession::PrimarySelection {
                start,
                current: _,
                active,
                target,
                mutation,
            }) => {
                self.state.drag = Some(ViewportDragSession::PrimarySelection {
                    start,
                    current: position,
                    active: active || start.distance(position) >= PRIMARY_NAV_THRESHOLD,
                    target,
                    mutation,
                });
            }
            Some(ViewportDragSession::Handle { mut session }) => {
                let camera = self.current_camera(scene);
                if let Some(target_pivot_world) = self.handles.update_drag(
                    &mut session,
                    &camera,
                    self.state.viewport.size,
                    position,
                ) {
                    feedback.transform_request = Some(ViewportTransformRequest {
                        primary: session.node_id(),
                        target_pivot_world,
                    });
                }
                self.state.drag = Some(ViewportDragSession::Handle { session });
            }
            None => {
                let (route, product_state) = self
                    .route_at_cursor(scene, position, false)
                    .map_err(|error| self.clear_hover_after_pointer_route_error(error))?;
                feedback.interaction_extract_stale |=
                    product_state == PointerBridgeProductState::Stale;
                feedback.hovered_axis = self.set_hover_route(route.as_ref());
            }
        }
        Ok(())
    }

    fn handle_selection_primary_pressed(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        selection_mutation: SelectionMutation,
        feedback: &mut ViewportFeedback,
    ) -> Result<(), UiTreeError> {
        let (route, current) = self.route_primary_pressed(scene, position, feedback)?;
        if !current {
            return Ok(());
        }
        self.begin_primary_selection(position, route, selection_mutation);
        Ok(())
    }

    fn handle_transform_primary_pressed(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        selection_mutation: SelectionMutation,
        feedback: &mut ViewportFeedback,
    ) -> Result<(), UiTreeError> {
        let (route, current) = self.route_primary_pressed(scene, position, feedback)?;
        if !current {
            return Ok(());
        }

        if let Some(ViewportPointerRoute::HandleAxis { axis, .. }) = route.as_ref() {
            if self.begin_handle_drag(scene, position, *axis) {
                feedback.hovered_axis = Some(*axis);
                return Ok(());
            }
        }
        self.begin_primary_selection(position, route, selection_mutation);
        Ok(())
    }

    fn route_primary_pressed(
        &mut self,
        scene: &Scene,
        position: Vec2,
        feedback: &mut ViewportFeedback,
    ) -> Result<(Option<ViewportPointerRoute>, bool), UiTreeError> {
        let (route, product_state) = self
            .route_at_cursor(scene, position, true)
            .map_err(|error| self.clear_hover_after_pointer_route_error(error))?;
        feedback.interaction_extract_stale |= product_state == PointerBridgeProductState::Stale;
        feedback.hovered_axis = self.set_hover_route(route.as_ref());
        Ok((route, product_state == PointerBridgeProductState::Current))
    }

    fn begin_primary_selection(
        &mut self,
        position: Vec2,
        route: Option<ViewportPointerRoute>,
        selection_mutation: SelectionMutation,
    ) {
        self.state.drag = Some(ViewportDragSession::PrimarySelection {
            start: position,
            current: position,
            active: false,
            target: route.as_ref().map(route_owner),
            mutation: selection_mutation,
        });
    }

    fn clear_hover_after_pointer_route_error(&mut self, error: UiTreeError) -> UiTreeError {
        self.set_hover_route(None);
        error
    }

    fn handle_left_released(&mut self, scene: &mut Scene) {
        match self.state.drag.take() {
            Some(ViewportDragSession::PrimarySelection {
                start,
                current,
                active,
                target,
                mutation,
            }) => {
                if active {
                    let selected = self
                        .pointer_bridge
                        .selectable_owners_in_rect(start, current);
                    let _ = self.select_nodes(scene, selected, mutation);
                } else {
                    let _ = self.select_nodes(scene, target, mutation);
                }
            }
            Some(ViewportDragSession::Handle { session }) => {
                self.state.drag = Some(ViewportDragSession::Handle { session });
                self.end_handle_drag();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
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

        fn handle_input(
            &mut self,
            input: &ViewportInput,
            ctx: &mut SceneModeCtx<'_>,
        ) -> InputOutcome {
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
        let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(before) =
            before
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
        let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(after) =
            after
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
        let source = include_str!("scene_viewport_controller_handle_input.rs");
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
}
