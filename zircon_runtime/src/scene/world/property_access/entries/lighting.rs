use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::EntityId;
use crate::scene::components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};

use super::super::super::World;

impl World {
    pub(super) fn visit_lighting_property_entries<F>(
        &self,
        entity: EntityId,
        visitor: &mut F,
    ) -> bool
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

        if let Some(light) = self.get::<AmbientLight>(entity) {
            push_entry!(
                "AmbientLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "AmbientLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "AmbientLight.affects_lightmapped_meshes",
                ScenePropertyValue::Bool(light.affects_lightmapped_meshes),
                false,
            );
        }
        if let Some(light) = self.get::<DirectionalLight>(entity) {
            push_entry!(
                "DirectionalLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push_entry!(
                "DirectionalLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "DirectionalLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
        }
        if let Some(light) = self.get::<PointLight>(entity) {
            push_entry!(
                "PointLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "PointLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "PointLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
        }
        if let Some(light) = self.get::<RectLight>(entity) {
            push_entry!(
                "RectLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "RectLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "RectLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
            push_entry!(
                "RectLight.size",
                ScenePropertyValue::Vec2(light.size.to_array()),
                true,
            );
        }
        if let Some(light) = self.get::<SpotLight>(entity) {
            push_entry!(
                "SpotLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push_entry!(
                "SpotLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "SpotLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "SpotLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
            push_entry!(
                "SpotLight.inner_angle_radians",
                ScenePropertyValue::Scalar(light.inner_angle_radians),
                true,
            );
            push_entry!(
                "SpotLight.outer_angle_radians",
                ScenePropertyValue::Scalar(light.outer_angle_radians),
                true,
            );
        }

        true
    }

    pub(super) fn lighting_property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        if self.contains_component::<AmbientLight>(entity) {
            capacity += 3;
        }
        if self.contains_component::<DirectionalLight>(entity) {
            capacity += 3;
        }
        if self.contains_component::<PointLight>(entity) {
            capacity += 3;
        }
        if self.contains_component::<RectLight>(entity) {
            capacity += 4;
        }
        if self.contains_component::<SpotLight>(entity) {
            capacity += 6;
        }
        capacity
    }
}
