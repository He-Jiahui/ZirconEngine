use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};
use crate::scene::{EntityId, SceneResult};

use super::super::super::World;
use super::super::value_conversion::{
    expect_bool, expect_scalar, expect_vec2, expect_vec3, missing_component_error,
    unknown_property_error,
};

impl World {
    pub(super) fn set_ambient_light_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(light) = self.get_mut::<AmbientLight>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "color" => {
                let next = expect_vec3(value, property_path)?;
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
            }
            [field] if field == "intensity" => {
                let next = expect_scalar(value, property_path)?;
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
            }
            [field] if field == "affectslightmappedmeshes" => {
                let next = expect_bool(value, property_path)?;
                if light.affects_lightmapped_meshes == next {
                    return Ok(false);
                }
                light.affects_lightmapped_meshes = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_directional_light_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(light) = self.get_mut::<DirectionalLight>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "direction" => {
                let next = expect_vec3(value, property_path)?;
                if light.direction == next {
                    return Ok(false);
                }
                light.direction = next;
            }
            [field] if field == "color" => {
                let next = expect_vec3(value, property_path)?;
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
            }
            [field] if field == "intensity" => {
                let next = expect_scalar(value, property_path)?;
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_point_light_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(light) = self.get_mut::<PointLight>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "color" => {
                let next = expect_vec3(value, property_path)?;
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
            }
            [field] if field == "intensity" => {
                let next = expect_scalar(value, property_path)?;
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
            }
            [field] if field == "range" => {
                let next = expect_scalar(value, property_path)?;
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_rect_light_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(light) = self.get_mut::<RectLight>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "color" => {
                let next = expect_vec3(value, property_path)?;
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
            }
            [field] if field == "intensity" => {
                let next = expect_scalar(value, property_path)?;
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
            }
            [field] if field == "range" => {
                let next = expect_scalar(value, property_path)?;
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
            }
            [field] if field == "size" => {
                let next = expect_vec2(value, property_path)?;
                if light.size == next {
                    return Ok(false);
                }
                light.size = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_spot_light_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(light) = self.get_mut::<SpotLight>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "direction" => {
                let next = expect_vec3(value, property_path)?;
                if light.direction == next {
                    return Ok(false);
                }
                light.direction = next;
            }
            [field] if field == "color" => {
                let next = expect_vec3(value, property_path)?;
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
            }
            [field] if field == "intensity" => {
                let next = expect_scalar(value, property_path)?;
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
            }
            [field] if field == "range" => {
                let next = expect_scalar(value, property_path)?;
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
            }
            [field] if field == "innerangleradians" => {
                let next = expect_scalar(value, property_path)?;
                if light.inner_angle_radians == next {
                    return Ok(false);
                }
                light.inner_angle_radians = next;
            }
            [field] if field == "outerangleradians" => {
                let next = expect_scalar(value, property_path)?;
                if light.outer_angle_radians == next {
                    return Ok(false);
                }
                light.outer_angle_radians = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }
}
