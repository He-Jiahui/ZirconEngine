use crate::core::framework::render::{
    RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot, RenderLayerSet,
    RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSpotLightSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::scene::components::{
    default_render_layer_mask, AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight,
};

use super::super::World;

impl World {
    pub(super) fn collect_volumetric_light_ids(&self, camera_layers: &RenderLayerSet) -> Vec<u64> {
        let mut light_ids = Vec::new();
        macro_rules! collect_volumetric_lights {
            ($ty:ty) => {
                if let Some(component_id) = self.registered_component_id::<$ty>() {
                    self.archetype_index.for_each_table_component::<$ty>(
                        component_id,
                        |entity, light| {
                            if !light.volumetric {
                                return;
                            }
                            if self.active_in_hierarchy(entity) == Some(true)
                                && self.entity_intersects_camera_layers(entity, camera_layers)
                            {
                                light_ids.push(entity);
                            }
                        },
                    );
                }
            };
        }
        collect_volumetric_lights!(DirectionalLight);
        collect_volumetric_lights!(PointLight);
        collect_volumetric_lights!(SpotLight);
        collect_volumetric_lights!(RectLight);
        light_ids.sort_unstable();
        light_ids.dedup();
        light_ids
    }

    pub(super) fn collect_ambient_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderAmbientLightSnapshot> {
        let Some(component_id) = self.registered_component_id::<AmbientLight>() else {
            return Vec::new();
        };
        let mut ambient_lights =
            Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index
            .for_each_table_component::<AmbientLight>(component_id, |entity, light| {
                if self.active_in_hierarchy(entity) != Some(true)
                    || !self.entity_intersects_camera_layers(entity, camera_layers)
                {
                    return;
                }
                ambient_lights.push((
                    entity,
                    RenderAmbientLightSnapshot {
                        color: light.color,
                        intensity: light.intensity,
                        renderer_degraded: false,
                        degradation_reason: None,
                    },
                ));
            });
        ambient_lights.sort_by_key(|(entity, _)| *entity);
        ambient_lights.into_iter().map(|(_, light)| light).collect()
    }

    pub(super) fn collect_directional_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderDirectionalLightSnapshot> {
        let Some(component_id) = self.registered_component_id::<DirectionalLight>() else {
            return Vec::new();
        };
        let mut directional_lights =
            Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index
            .for_each_table_component::<DirectionalLight>(component_id, |entity, light| {
                if self.active_in_hierarchy(entity) != Some(true)
                    || !self.entity_intersects_camera_layers(entity, camera_layers)
                {
                    return;
                }
                directional_lights.push(RenderDirectionalLightSnapshot {
                    node_id: entity,
                    light_id: entity,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        self.render_layer_mask(entity)
                            .unwrap_or(default_render_layer_mask()),
                    ),
                    direction: light.direction,
                    color: light.color,
                    intensity: light.intensity,
                    mobility: self.mobility(entity).unwrap_or(Mobility::Dynamic),
                    shadow: None,
                });
            });
        directional_lights.sort_by_key(|light| light.node_id);
        directional_lights
    }

    pub(super) fn collect_point_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderPointLightSnapshot> {
        let Some(component_id) = self.registered_component_id::<PointLight>() else {
            return Vec::new();
        };
        let mut point_lights = Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index.for_each_table_component::<PointLight>(
            component_id,
            |entity, light| {
                if self.active_in_hierarchy(entity) != Some(true)
                    || !self.entity_intersects_camera_layers(entity, camera_layers)
                {
                    return;
                }
                point_lights.push(RenderPointLightSnapshot {
                    node_id: entity,
                    light_id: entity,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        self.render_layer_mask(entity)
                            .unwrap_or(default_render_layer_mask()),
                    ),
                    position: self.world_transform(entity).unwrap_or_default().translation,
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    mobility: self.mobility(entity).unwrap_or(Mobility::Dynamic),
                    shadow: None,
                });
            },
        );
        point_lights.sort_by_key(|light| light.node_id);
        point_lights
    }

    pub(super) fn collect_rect_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderRectLightSnapshot> {
        let Some(component_id) = self.registered_component_id::<RectLight>() else {
            return Vec::new();
        };
        let mut rect_lights = Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index.for_each_table_component::<RectLight>(
            component_id,
            |entity, light| {
                if self.active_in_hierarchy(entity) != Some(true)
                    || !self.entity_intersects_camera_layers(entity, camera_layers)
                {
                    return;
                }
                let transform = self.world_transform(entity).unwrap_or_default();
                rect_lights.push(RenderRectLightSnapshot {
                    node_id: entity,
                    light_id: entity,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        self.render_layer_mask(entity)
                            .unwrap_or(default_render_layer_mask()),
                    ),
                    position: transform.translation,
                    direction: transform.forward(),
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    size: light.size,
                    shadow: None,
                    renderer_degraded: true,
                    degradation_reason: Some(
                        "rect light renderer shading is not implemented yet".to_string(),
                    ),
                });
            },
        );
        rect_lights.sort_by_key(|light| light.node_id);
        rect_lights
    }

    pub(super) fn collect_spot_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderSpotLightSnapshot> {
        let Some(component_id) = self.registered_component_id::<SpotLight>() else {
            return Vec::new();
        };
        let mut spot_lights = Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index.for_each_table_component::<SpotLight>(
            component_id,
            |entity, light| {
                if self.active_in_hierarchy(entity) != Some(true)
                    || !self.entity_intersects_camera_layers(entity, camera_layers)
                {
                    return;
                }
                spot_lights.push(RenderSpotLightSnapshot {
                    node_id: entity,
                    light_id: entity,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        self.render_layer_mask(entity)
                            .unwrap_or(default_render_layer_mask()),
                    ),
                    position: self.world_transform(entity).unwrap_or_default().translation,
                    direction: light.direction,
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    inner_angle_radians: light.inner_angle_radians,
                    outer_angle_radians: light.outer_angle_radians,
                    mobility: self.mobility(entity).unwrap_or(Mobility::Dynamic),
                    shadow: None,
                });
            },
        );
        spot_lights.sort_by_key(|light| light.node_id);
        spot_lights
    }
}
