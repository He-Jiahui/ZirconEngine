use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::CameraComponent;
use crate::scene::{EntityId, SceneResult};

use super::super::super::World;
use super::super::value_conversion::{
    expect_scalar, missing_component_error, unknown_property_error,
};

impl World {
    pub(super) fn set_camera_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(camera) = self.get_mut::<CameraComponent>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "fovyradians" => {
                let scalar = expect_scalar(value, property_path)?;
                if camera.fov_y_radians == scalar {
                    return Ok(false);
                }
                camera.fov_y_radians = scalar;
            }
            [field] if field == "znear" => {
                let scalar = expect_scalar(value, property_path)?;
                if camera.z_near == scalar {
                    return Ok(false);
                }
                camera.z_near = scalar;
            }
            [field] if field == "zfar" => {
                let scalar = expect_scalar(value, property_path)?;
                if camera.z_far == scalar {
                    return Ok(false);
                }
                camera.z_far = scalar;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }
}
