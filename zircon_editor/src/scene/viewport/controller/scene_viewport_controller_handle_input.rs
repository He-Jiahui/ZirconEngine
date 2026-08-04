use crate::scene::modes::{InputOutcome, SceneModeCtx, SceneModeInputEffect};
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::ViewportInput;
use crate::scene::viewport::{ViewportFeedback, ViewportTransformPreview};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec2;

use crate::scene::viewport::pointer::ViewportPointerRoute;

use super::{
    SceneViewportController, constants::PRIMARY_NAV_THRESHOLD,
    scene_viewport_controller_pointer_route::route_owner,
    viewport_drag_session::ViewportDragSession,
};

impl SceneViewportController {
    pub(crate) fn handle_input(
        &mut self,
        scene: &mut Scene,
        input: ViewportInput,
    ) -> ViewportFeedback {
        let mut feedback = ViewportFeedback::default();

        // A resize maintains the controller's viewport invariant even when a mode owns the
        // event. All other builtin interaction runs only after the mode stack passes through.
        if let ViewportInput::Resized(size) = &input {
            self.apply_viewport_size(*size);
        }
        let (outcome, effect, overlay_invalidated) = self.dispatch_scene_mode_input(&input);
        if overlay_invalidated {
            self.interaction_extract.invalidate();
        }
        if outcome == InputOutcome::Consumed {
            if let Some(effect) = effect {
                self.apply_scene_mode_input_effect(scene, effect, &mut feedback);
            }
            return feedback;
        }

        match input {
            ViewportInput::Resized(_) => {}
            ViewportInput::PointerMoved(_)
            | ViewportInput::LeftPressed { .. }
            | ViewportInput::LeftReleased => {}
            ViewportInput::RightPressed(position) => {
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                self.state.drag = Some(ViewportDragSession::Orbit { last: position });
            }
            ViewportInput::RightReleased => {
                self.state.drag = None;
            }
            ViewportInput::MiddlePressed(position) => {
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                self.state.drag = Some(ViewportDragSession::Pan { last: position });
            }
            ViewportInput::MiddleReleased => {
                self.state.drag = None;
            }
            ViewportInput::Scrolled(delta) => {
                if self.state.camera.is_none() {
                    self.reset_camera_from_scene(Some(scene));
                }
                feedback.camera_updated = self.apply_zoom(delta);
            }
        }

        feedback
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
    ) {
        match effect {
            SceneModeInputEffect::PointerMoved(position) => {
                self.handle_pointer_moved(scene, position, feedback);
            }
            SceneModeInputEffect::PrimaryPressed {
                position,
                allow_handle_drag,
                selection_mutation,
            } => {
                self.handle_primary_pressed(
                    scene,
                    position,
                    allow_handle_drag,
                    selection_mutation,
                    feedback,
                );
            }
            SceneModeInputEffect::PrimaryReleased => self.handle_left_released(scene),
        }
    }

    fn handle_pointer_moved(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        feedback: &mut ViewportFeedback,
    ) {
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
                if let Some(transform) = self.handles.update_drag(
                    &mut session,
                    &camera,
                    self.state.viewport.size,
                    position,
                ) {
                    feedback.transform_preview = Some(ViewportTransformPreview {
                        node_id: session.node_id(),
                        transform,
                    });
                }
                self.state.drag = Some(ViewportDragSession::Handle { session });
            }
            None => {
                let route = self.route_at_cursor(scene, position, false);
                feedback.hovered_axis = self.set_hover_route(route.as_ref());
            }
        }
    }

    fn handle_primary_pressed(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        allow_handle_drag: bool,
        selection_mutation: SelectionMutation,
        feedback: &mut ViewportFeedback,
    ) {
        let route = self.route_at_cursor(scene, position, true);
        feedback.hovered_axis = self.set_hover_route(route.as_ref());

        if allow_handle_drag {
            if let Some(ViewportPointerRoute::HandleAxis { axis, .. }) = route.as_ref() {
                if self.begin_handle_drag(scene, position, *axis) {
                    feedback.hovered_axis = Some(*axis);
                    return;
                }
            }
        }

        self.state.drag = Some(ViewportDragSession::PrimarySelection {
            start: position,
            current: position,
            active: false,
            target: route.as_ref().map(route_owner),
            mutation: selection_mutation,
        });
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
    use crate::scene::modes::{EditorSceneMode, ViewportOverlayBuilder};
    use zircon_runtime_interface::math::UVec2;

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
                .push(Box::new(ConsumingPointerMode::new()), &mut mode_ctx)
                .unwrap();
        }

        let feedback = controller.handle_input(
            &mut Scene::new(),
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        );

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
                .push(Box::new(ConsumingPointerMode::new()), &mut mode_ctx)
                .unwrap();
        }
        let scene = Scene::new();
        let settings = SceneViewportSettings::default();
        let camera = ViewportCameraSnapshot::default();
        let viewport = UVec2::new(1280, 720);
        let before = controller.interaction_extract.resolve_for_pointer(
            &scene,
            None,
            &settings,
            &camera,
            viewport,
            Vec::new,
            Vec::new,
        );

        controller.handle_input(
            &mut Scene::new(),
            ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
        );

        let after = controller.interaction_extract.resolve_for_pointer(
            &scene,
            None,
            &settings,
            &camera,
            viewport,
            Vec::new,
            Vec::new,
        );
        assert!(!Arc::ptr_eq(&before, &after));
    }

    #[test]
    fn transform_mode_input_only_publishes_transaction_preview_requests() {
        let source = include_str!("scene_viewport_controller_handle_input.rs");
        let production_source = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(production, _)| production);

        assert!(production_source.contains("feedback.transform_preview"));
        assert!(!production_source.contains("scene.update_transform("));
    }
}
