use crate::scene::viewport::{ProjectionMode, ViewOrientation};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Transform, Vec3};

use super::SceneViewportController;

const FRAME_DISTANCE: f32 = 6.0;
const FRAME_ORTHO_SIZE: f32 = 2.5;
const FRAME_PERSPECTIVE_PADDING: f32 = 1.15;
const FRAME_ORTHO_PADDING: f32 = 1.15;

#[derive(Clone, Copy, Debug)]
struct SelectionFrame {
    target: Vec3,
    radius: f32,
}

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn frame_selection(
        &mut self,
        scene: &Scene,
    ) -> bool {
        let Some(frame) =
            selection_frame(scene, self.state.selection.active_items().iter().copied())
        else {
            return false;
        };

        let mut camera = self.current_camera(scene);
        let offset = camera.transform.translation - frame.target;
        let direction = if offset.length_squared() > f32::EPSILON {
            offset.normalize_or_zero()
        } else {
            Vec3::new(0.6, 0.45, 1.0).normalize_or_zero()
        };
        let distance = offset.length().max(FRAME_DISTANCE).max(
            frame.radius * FRAME_PERSPECTIVE_PADDING
                / (camera.fov_y_radians * 0.5).sin().abs().max(f32::EPSILON),
        );
        camera.transform =
            Transform::looking_at(frame.target + direction * distance, frame.target, Vec3::Y);
        if camera.projection_mode == ProjectionMode::Orthographic {
            camera.ortho_size = FRAME_ORTHO_SIZE.max(frame.radius * FRAME_ORTHO_PADDING);
        }
        camera.apply_viewport_size(self.state.viewport.size);

        self.state.camera = Some(camera);
        self.state.orbit_target = frame.target;
        self.state.orbit_controller.set_target(frame.target);
        self.state.settings.view_orientation = ViewOrientation::User;
        true
    }
}

fn selection_frame(
    scene: &Scene,
    selected: impl IntoIterator<Item = u64>,
) -> Option<SelectionFrame> {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);

    for entity in selected {
        let Some(position) = SceneViewportController::selected_world_position(scene, Some(entity))
        else {
            continue;
        };
        if !position.is_finite() {
            continue;
        }
        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    if !minimum.is_finite() || !maximum.is_finite() {
        return None;
    }

    let target = (minimum + maximum) * 0.5;
    Some(SelectionFrame {
        target,
        radius: maximum.distance(target),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::viewport::SceneViewportController;
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime_interface::math::UVec2;

    #[test]
    fn frame_selection_centers_the_active_multiselection_instead_of_the_primary_item() {
        let mut scene = Scene::new();
        let left = scene
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        let right = scene
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        scene
            .update_transform(
                left,
                Transform {
                    translation: Vec3::new(-8.0, 0.0, 0.0),
                    ..Transform::default()
                },
            )
            .unwrap();
        scene
            .update_transform(
                right,
                Transform {
                    translation: Vec3::new(8.0, 0.0, 0.0),
                    ..Transform::default()
                },
            )
            .unwrap();

        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        controller
            .selection_mut()
            .replace_active([left, right], Some(left));

        assert!(controller.frame_selection(&scene));
        assert_eq!(controller.orbit_target(), Vec3::ZERO);
        let camera = controller.current_camera(&scene);
        let minimum_distance = 8.0 * FRAME_PERSPECTIVE_PADDING
            / (camera.fov_y_radians * 0.5).sin().abs().max(f32::EPSILON);
        assert!(
            camera.transform.translation.distance(Vec3::ZERO) >= minimum_distance,
            "frame selection must expand the camera enough to contain the multi-selection bounds"
        );
    }
}
