use crate::{ViewportFeedback, ViewportInput};
use crate::scene::viewport::SceneViewportTool;
use zircon_runtime::core::math::Vec2;
use zircon_runtime::scene::Scene;

use crate::scene::viewport::pointer::ViewportPointerRoute;

use super::{
    constants::PRIMARY_NAV_THRESHOLD, scene_viewport_controller_pointer_route::route_owner,
    viewport_drag_session::ViewportDragSession, SceneViewportController,
};

impl SceneViewportController {
    pub(crate) fn handle_input(
        &mut self,
        scene: &mut Scene,
        input: ViewportInput,
    ) -> ViewportFeedback {
        let mut feedback = ViewportFeedback::default();

        match input {
            ViewportInput::Resized(size) => {
                self.apply_viewport_size(size);
            }
            ViewportInput::PointerMoved(position) => {
                self.handle_pointer_moved(scene, position, &mut feedback);
            }
            ViewportInput::LeftPressed(position) => {
                self.handle_left_pressed(scene, position, &mut feedback);
            }
            ViewportInput::LeftReleased => {
                self.handle_left_released(scene);
            }
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
            Some(ViewportDragSession::PrimaryNavigation {
                start,
                active,
                target,
            }) => {
                self.state.drag = Some(ViewportDragSession::PrimaryNavigation {
                    start,
                    active: active || start.distance(position) >= PRIMARY_NAV_THRESHOLD,
                    target,
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
                    let node_id = session.node_id();
                    if scene.update_transform(node_id, transform).is_ok() {
                        if node_id == scene.active_camera() {
                            if let Some(camera_state) = self.state.camera.as_mut() {
                                camera_state.transform = transform;
                            }
                        }
                        feedback.transformed_node = Some(node_id);
                    }
                }
                self.state.drag = Some(ViewportDragSession::Handle { session });
            }
            None => {
                let route = self.route_at_cursor(scene, position, false);
                feedback.hovered_axis = self.set_hover_route(route.as_ref());
            }
        }
    }

    fn handle_left_pressed(
        &mut self,
        scene: &mut Scene,
        position: Vec2,
        feedback: &mut ViewportFeedback,
    ) {
        let route = self.route_at_cursor(scene, position, true);
        feedback.hovered_axis = self.set_hover_route(route.as_ref());

        if let Some(ViewportPointerRoute::HandleAxis { axis, .. }) = route.as_ref() {
            if self.state.settings.tool != SceneViewportTool::Drag
                && self.begin_handle_drag(scene, position, *axis)
            {
                feedback.hovered_axis = Some(*axis);
                return;
            }
        }

        self.state.drag = Some(ViewportDragSession::PrimaryNavigation {
            start: position,
            active: false,
            target: route.as_ref().map(route_owner),
        });
    }

    fn handle_left_released(&mut self, scene: &mut Scene) {
        match self.state.drag.take() {
            Some(ViewportDragSession::PrimaryNavigation { active, target, .. }) if !active => {
                let _ = self.select_node(scene, target);
            }
            Some(ViewportDragSession::Handle { session }) => {
                self.state.drag = Some(ViewportDragSession::Handle { session });
                self.end_handle_drag();
            }
            _ => {}
        }
    }
}
