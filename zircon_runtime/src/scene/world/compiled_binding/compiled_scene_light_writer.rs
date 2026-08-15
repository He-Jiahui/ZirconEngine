use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};
use crate::scene::world::{SceneError, SceneResult, World};

use super::compiled_scene_camera_light_fields::CompiledLightProperty;
use super::property_path::CompiledScenePropertyWriter;

impl World {
    pub(super) fn write_compiled_light_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        property: CompiledLightProperty,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let changed = match property {
            CompiledLightProperty::AmbientColor => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<AmbientLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "AmbientLight");
                };
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
                true
            }
            CompiledLightProperty::AmbientIntensity => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<AmbientLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "AmbientLight");
                };
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
                true
            }
            CompiledLightProperty::AmbientAffectsLightmappedMeshes => {
                let next = Self::compiled_property_expect_bool(value, target.property_path())?;
                let Some(light) = self.get_mut::<AmbientLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "AmbientLight");
                };
                if light.affects_lightmapped_meshes == next {
                    return Ok(false);
                }
                light.affects_lightmapped_meshes = next;
                true
            }
            CompiledLightProperty::DirectionalDirection => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<DirectionalLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "DirectionalLight");
                };
                if light.direction == next {
                    return Ok(false);
                }
                light.direction = next;
                true
            }
            CompiledLightProperty::DirectionalColor => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<DirectionalLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "DirectionalLight");
                };
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
                true
            }
            CompiledLightProperty::DirectionalIntensity => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<DirectionalLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "DirectionalLight");
                };
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
                true
            }
            CompiledLightProperty::PointColor => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<PointLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "PointLight");
                };
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
                true
            }
            CompiledLightProperty::PointIntensity => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<PointLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "PointLight");
                };
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
                true
            }
            CompiledLightProperty::PointRange => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<PointLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "PointLight");
                };
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
                true
            }
            CompiledLightProperty::RectColor => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<RectLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "RectLight");
                };
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
                true
            }
            CompiledLightProperty::RectIntensity => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<RectLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "RectLight");
                };
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
                true
            }
            CompiledLightProperty::RectRange => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<RectLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "RectLight");
                };
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
                true
            }
            CompiledLightProperty::RectSize => {
                let next = Self::compiled_property_expect_vec2(value, target.property_path())?;
                let Some(light) = self.get_mut::<RectLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "RectLight");
                };
                if light.size == next {
                    return Ok(false);
                }
                light.size = next;
                true
            }
            CompiledLightProperty::SpotDirection => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.direction == next {
                    return Ok(false);
                }
                light.direction = next;
                true
            }
            CompiledLightProperty::SpotColor => {
                let next = Self::compiled_property_expect_vec3(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.color == next {
                    return Ok(false);
                }
                light.color = next;
                true
            }
            CompiledLightProperty::SpotIntensity => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.intensity == next {
                    return Ok(false);
                }
                light.intensity = next;
                true
            }
            CompiledLightProperty::SpotRange => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.range == next {
                    return Ok(false);
                }
                light.range = next;
                true
            }
            CompiledLightProperty::SpotInnerAngleRadians => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.inner_angle_radians == next {
                    return Ok(false);
                }
                light.inner_angle_radians = next;
                true
            }
            CompiledLightProperty::SpotOuterAngleRadians => {
                let next = Self::compiled_property_expect_scalar(value, target.property_path())?;
                let Some(light) = self.get_mut::<SpotLight>(target.entity()) else {
                    return Self::missing_compiled_light(target, "SpotLight");
                };
                if light.outer_angle_radians == next {
                    return Ok(false);
                }
                light.outer_angle_radians = next;
                true
            }
        };
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }

    fn missing_compiled_light(
        target: &CompiledScenePropertyWriter,
        component: &'static str,
    ) -> SceneResult<bool> {
        Err(SceneError::MissingRequiredComponent {
            operation: "write compiled light property",
            entity: target.entity(),
            component,
        })
    }
}
