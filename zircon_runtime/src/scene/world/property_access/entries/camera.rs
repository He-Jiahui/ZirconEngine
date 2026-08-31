use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::EntityId;
use crate::scene::components::CameraComponent;

use super::super::super::World;

impl World {
    pub(super) fn visit_camera_property_entries<F>(&self, entity: EntityId, visitor: &mut F) -> bool
    where
        F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
    {
        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                let mut build_value = || $value;
                if !visitor($path, &mut build_value, $animatable) {
                    return false;
                }
            };
        }

        if let Some(camera) = self.get::<CameraComponent>(entity) {
            push_entry!(
                "Camera.fov_y_radians",
                ScenePropertyValue::Scalar(camera.fov_y_radians),
                true,
            );
            push_entry!(
                "Camera.z_near",
                ScenePropertyValue::Scalar(camera.z_near),
                true,
            );
            push_entry!(
                "Camera.z_far",
                ScenePropertyValue::Scalar(camera.z_far),
                true,
            );
        }

        true
    }

    pub(super) fn camera_property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        if self.contains_component::<CameraComponent>(entity) {
            capacity += 3;
        }
        capacity
    }
}
