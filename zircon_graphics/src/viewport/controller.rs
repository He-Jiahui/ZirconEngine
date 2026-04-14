use crate::types::{GizmoAxis, ViewportFeedback, ViewportInput, ViewportState};
use zircon_math::{perspective, view_matrix, Transform, UVec2, Vec2, Vec3};
use zircon_scene::{CameraComponent, NodeId, Scene};

#[derive(Clone, Debug)]
pub struct ViewportController {
    viewport: ViewportState,
    orbit_target: Vec3,
    hovered_axis: Option<GizmoAxis>,
    drag: Option<DragState>,
}

#[derive(Clone, Debug)]
enum DragState {
    Orbit {
        last: Vec2,
    },
    Pan {
        last: Vec2,
    },
    Translate {
        node_id: NodeId,
        axis: GizmoAxis,
        last: Vec2,
    },
}

impl ViewportController {
    pub fn new(viewport: ViewportState) -> Self {
        Self {
            viewport,
            orbit_target: Vec3::ZERO,
            hovered_axis: None,
            drag: None,
        }
    }

    pub fn viewport(&self) -> &ViewportState {
        &self.viewport
    }

    pub fn hovered_axis(&self) -> Option<GizmoAxis> {
        self.hovered_axis
    }

    pub fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit_target = target;
    }

    pub fn handle_input(&mut self, scene: &mut Scene, input: ViewportInput) -> ViewportFeedback {
        let mut feedback = ViewportFeedback::default();

        match input {
            ViewportInput::Resized(size) => {
                self.viewport = ViewportState::new(size);
            }
            ViewportInput::PointerMoved(position) => match self.drag.take() {
                Some(DragState::Orbit { last }) => {
                    self.apply_orbit(scene, last, position);
                    feedback.camera_updated = true;
                    self.drag = Some(DragState::Orbit { last: position });
                }
                Some(DragState::Pan { last }) => {
                    self.apply_pan(scene, last, position);
                    feedback.camera_updated = true;
                    self.drag = Some(DragState::Pan { last: position });
                }
                Some(DragState::Translate {
                    node_id,
                    axis,
                    last,
                }) => {
                    if self.apply_translation(scene, node_id, axis, last, position) {
                        feedback.transformed_node = Some(node_id);
                    }
                    self.drag = Some(DragState::Translate {
                        node_id,
                        axis,
                        last: position,
                    });
                }
                None => {
                    self.hovered_axis = self.pick_axis(scene, position);
                    feedback.hovered_axis = self.hovered_axis;
                }
            },
            ViewportInput::LeftPressed(position) => {
                self.hovered_axis = self.pick_axis(scene, position);
                if let (Some(axis), Some(node_id)) = (self.hovered_axis, scene.selected_node()) {
                    self.drag = Some(DragState::Translate {
                        node_id,
                        axis,
                        last: position,
                    });
                }
                feedback.hovered_axis = self.hovered_axis;
            }
            ViewportInput::LeftReleased => {
                self.drag = None;
            }
            ViewportInput::RightPressed(position) => {
                self.drag = Some(DragState::Orbit { last: position });
            }
            ViewportInput::RightReleased => {
                self.drag = None;
            }
            ViewportInput::MiddlePressed(position) => {
                self.drag = Some(DragState::Pan { last: position });
            }
            ViewportInput::MiddleReleased => {
                self.drag = None;
            }
            ViewportInput::Scrolled(delta) => {
                self.apply_zoom(scene, delta);
                feedback.camera_updated = true;
            }
        }

        feedback
    }

    fn apply_orbit(&self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let delta = (current - previous) * 0.01;
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let offset = camera.transform.translation - self.orbit_target;
        let distance = offset.length().max(0.001);
        let mut yaw = offset.x.atan2(offset.z);
        let horizontal = (offset.x * offset.x + offset.z * offset.z)
            .sqrt()
            .max(0.001);
        let mut pitch = offset.y.atan2(horizontal);

        yaw -= delta.x;
        pitch = (pitch + delta.y).clamp(-1.4, 1.4);

        let next_offset = Vec3::new(
            distance * pitch.cos() * yaw.sin(),
            distance * pitch.sin(),
            distance * pitch.cos() * yaw.cos(),
        );
        let transform =
            Transform::looking_at(self.orbit_target + next_offset, self.orbit_target, Vec3::Y);
        scene.update_transform(camera.id, transform);
    }

    fn apply_pan(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let delta = current - previous;
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let distance = (camera.transform.translation - self.orbit_target)
            .length()
            .max(0.5);
        let world_per_pixel = distance * 0.0015;
        let translation = (-camera.transform.right() * delta.x + camera.transform.up() * delta.y)
            * world_per_pixel;
        let transform = Transform {
            translation: camera.transform.translation + translation,
            ..camera.transform
        };
        self.orbit_target += translation;
        scene.update_transform(camera.id, transform);
    }

    fn apply_zoom(&self, scene: &mut Scene, delta: f32) {
        let Some(camera) = scene.find_node(scene.active_camera()).cloned() else {
            return;
        };
        let direction = camera.transform.forward();
        let distance = (camera.transform.translation - self.orbit_target)
            .length()
            .max(0.25);
        let step = (distance * 0.15 * delta.signum()).min(distance - 0.25);
        let transform = Transform {
            translation: camera.transform.translation + direction * step,
            ..camera.transform
        };
        scene.update_transform(camera.id, transform);
    }

    fn apply_translation(
        &self,
        scene: &mut Scene,
        node_id: NodeId,
        axis: GizmoAxis,
        previous: Vec2,
        current: Vec2,
    ) -> bool {
        let Some(node) = scene.find_node(node_id).cloned() else {
            return false;
        };
        let Some(camera) = scene.find_node(scene.active_camera()) else {
            return false;
        };
        let Some(camera_component) = camera.camera.as_ref() else {
            return false;
        };

        let origin = node.transform.translation;
        let Some(start) = project_point(
            origin,
            camera.transform,
            camera_component,
            self.viewport.size,
        ) else {
            return false;
        };
        let Some(end) = project_point(
            origin + axis_vector(axis),
            camera.transform,
            camera_component,
            self.viewport.size,
        ) else {
            return false;
        };

        let direction = (end - start).normalize_or_zero();
        if direction.length_squared() <= f32::EPSILON {
            return false;
        }

        let distance = (camera.transform.translation - origin).length().max(0.5);
        let world_per_pixel = distance * (camera_component.fov_y_radians * 0.5).tan()
            / self.viewport.size.y.max(1) as f32
            * 2.0;
        let delta_pixels = (current - previous).dot(direction);
        let transform = Transform {
            translation: node.transform.translation
                + axis_vector(axis) * delta_pixels * world_per_pixel,
            ..node.transform
        };
        scene.update_transform(node_id, transform);
        true
    }

    fn pick_axis(&self, scene: &Scene, cursor: Vec2) -> Option<GizmoAxis> {
        let selected_id = scene.selected_node()?;
        let selected = scene.find_node(selected_id)?;
        let camera = scene.find_node(scene.active_camera())?;
        let camera_component = camera.camera.as_ref()?;

        let mut best = None;
        let mut best_distance = 12.0_f32;
        for axis in [GizmoAxis::X, GizmoAxis::Y, GizmoAxis::Z] {
            let Some(start) = project_point(
                selected.transform.translation,
                camera.transform,
                camera_component,
                self.viewport.size,
            ) else {
                continue;
            };
            let Some(end) = project_point(
                selected.transform.translation + axis_vector(axis),
                camera.transform,
                camera_component,
                self.viewport.size,
            ) else {
                continue;
            };
            let distance = distance_to_segment(cursor, start, end);
            if distance < best_distance {
                best_distance = distance;
                best = Some(axis);
            }
        }
        best
    }
}

fn axis_vector(axis: GizmoAxis) -> Vec3 {
    match axis {
        GizmoAxis::X => Vec3::X,
        GizmoAxis::Y => Vec3::Y,
        GizmoAxis::Z => Vec3::Z,
    }
}

fn project_point(
    world: Vec3,
    camera_transform: Transform,
    camera: &CameraComponent,
    viewport: UVec2,
) -> Option<Vec2> {
    let viewport = UVec2::new(viewport.x.max(1), viewport.y.max(1));
    let aspect = viewport.x as f32 / viewport.y as f32;
    let clip = perspective(camera.fov_y_radians, aspect, camera.z_near, camera.z_far)
        * view_matrix(camera_transform)
        * world.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some(Vec2::new(
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
    ))
}

fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_scene::NodeKind;

    #[test]
    fn viewport_resized_is_clamped() {
        let mut controller = ViewportController::new(ViewportState::new(UVec2::new(10, 10)));
        let mut scene = Scene::new();
        controller.handle_input(&mut scene, ViewportInput::Resized(UVec2::ZERO));

        assert_eq!(controller.viewport().size, UVec2::ONE);
    }

    #[test]
    fn pointer_move_without_selection_is_safe() {
        let mut controller = ViewportController::new(ViewportState::new(UVec2::new(1280, 720)));
        let mut scene = Scene::new();
        let camera = scene
            .nodes()
            .iter()
            .find(|node| matches!(&node.kind, NodeKind::Camera))
            .unwrap()
            .id;
        scene.set_selected(Some(camera));

        let _ = controller.handle_input(
            &mut scene,
            ViewportInput::PointerMoved(Vec2::new(640.0, 360.0)),
        );
    }

    #[test]
    fn translate_drag_moves_selected_cube() {
        let mut controller = ViewportController::new(ViewportState::new(UVec2::new(1280, 720)));
        let mut scene = Scene::new();
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id;
        let camera = scene.find_node(scene.active_camera()).unwrap().clone();
        let camera_component = camera.camera.as_ref().unwrap();
        let cube_origin = scene.find_node(cube).unwrap().transform.translation;
        scene.set_selected(Some(cube));

        let start = project_point(
            cube_origin,
            camera.transform,
            camera_component,
            controller.viewport().size,
        )
        .unwrap();
        let end = project_point(
            cube_origin + Vec3::X,
            camera.transform,
            camera_component,
            controller.viewport().size,
        )
        .unwrap();
        let press = start + (end - start) * 0.5;
        let drag_to = press + (end - start).normalize_or_zero() * 64.0;

        let pressed = controller.handle_input(&mut scene, ViewportInput::LeftPressed(press));
        let moved = controller.handle_input(&mut scene, ViewportInput::PointerMoved(drag_to));
        let _ = controller.handle_input(&mut scene, ViewportInput::LeftReleased);

        assert_eq!(pressed.hovered_axis, Some(GizmoAxis::X));
        assert_eq!(moved.transformed_node, Some(cube));
        assert!(scene.find_node(cube).unwrap().transform.translation.x > cube_origin.x);
    }
}
