use crate::core::framework::render::{
    LightShadowSettings, RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot,
    RenderLayerSet, RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSpotLightSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::scene::components::{
    default_render_layer_mask, AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight,
};

use super::super::World;

pub(super) struct CollectedRenderLights {
    pub(super) ambient_lights: Vec<RenderAmbientLightSnapshot>,
    pub(super) directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub(super) point_lights: Vec<RenderPointLightSnapshot>,
    pub(super) rect_lights: Vec<RenderRectLightSnapshot>,
    pub(super) spot_lights: Vec<RenderSpotLightSnapshot>,
    pub(super) volumetric_light_ids: Vec<u64>,
}

impl World {
    pub(super) fn collect_render_lights(
        &self,
        camera_layers: &RenderLayerSet,
        include_volumetric_ids: bool,
    ) -> CollectedRenderLights {
        let mut volumetric_light_ids = Vec::new();
        let ambient_lights = self.collect_ambient_lights(camera_layers);
        let directional_lights = self.collect_directional_lights_with_volumetric_ids(
            camera_layers,
            include_volumetric_ids.then_some(&mut volumetric_light_ids),
        );
        let point_lights = self.collect_point_lights_with_volumetric_ids(
            camera_layers,
            include_volumetric_ids.then_some(&mut volumetric_light_ids),
        );
        let rect_lights = self.collect_rect_lights_with_volumetric_ids(
            camera_layers,
            include_volumetric_ids.then_some(&mut volumetric_light_ids),
        );
        let spot_lights = self.collect_spot_lights_with_volumetric_ids(
            camera_layers,
            include_volumetric_ids.then_some(&mut volumetric_light_ids),
        );
        volumetric_light_ids.sort_unstable();
        volumetric_light_ids.dedup();

        CollectedRenderLights {
            ambient_lights,
            directional_lights,
            point_lights,
            rect_lights,
            spot_lights,
            volumetric_light_ids,
        }
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
                        affects_lightmapped_meshes: light.affects_lightmapped_meshes,
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
        self.collect_directional_lights_with_volumetric_ids(camera_layers, None)
    }

    fn collect_directional_lights_with_volumetric_ids(
        &self,
        camera_layers: &RenderLayerSet,
        mut volumetric_light_ids: Option<&mut Vec<u64>>,
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
                if light.volumetric {
                    if let Some(light_ids) = volumetric_light_ids.as_deref_mut() {
                        light_ids.push(entity);
                    }
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
                    shadow: extracted_shadow_settings(light.casts_shadow),
                });
            });
        directional_lights.sort_by_key(|light| light.node_id);
        directional_lights
    }

    pub(super) fn collect_point_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderPointLightSnapshot> {
        self.collect_point_lights_with_volumetric_ids(camera_layers, None)
    }

    fn collect_point_lights_with_volumetric_ids(
        &self,
        camera_layers: &RenderLayerSet,
        mut volumetric_light_ids: Option<&mut Vec<u64>>,
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
                if light.volumetric {
                    if let Some(light_ids) = volumetric_light_ids.as_deref_mut() {
                        light_ids.push(entity);
                    }
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
                    shadow: extracted_shadow_settings(light.casts_shadow),
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
        self.collect_rect_lights_with_volumetric_ids(camera_layers, None)
    }

    fn collect_rect_lights_with_volumetric_ids(
        &self,
        camera_layers: &RenderLayerSet,
        mut volumetric_light_ids: Option<&mut Vec<u64>>,
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
                if light.volumetric {
                    if let Some(light_ids) = volumetric_light_ids.as_deref_mut() {
                        light_ids.push(entity);
                    }
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
                        if light.casts_shadow {
                            "rect light shading and shadow rendering are not implemented yet"
                        } else {
                            "rect light renderer shading is not implemented yet"
                        }
                        .to_string(),
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
        self.collect_spot_lights_with_volumetric_ids(camera_layers, None)
    }

    fn collect_spot_lights_with_volumetric_ids(
        &self,
        camera_layers: &RenderLayerSet,
        mut volumetric_light_ids: Option<&mut Vec<u64>>,
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
                if light.volumetric {
                    if let Some(light_ids) = volumetric_light_ids.as_deref_mut() {
                        light_ids.push(entity);
                    }
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
                    shadow: extracted_shadow_settings(light.casts_shadow),
                });
            },
        );
        spot_lights.sort_by_key(|light| light.node_id);
        spot_lights
    }
}

fn extracted_shadow_settings(casts_shadow: bool) -> Option<LightShadowSettings> {
    casts_shadow.then(|| LightShadowSettings {
        casts_shadow: true,
        ..LightShadowSettings::default()
    })
}

#[cfg(test)]
#[path = "tests/lights.rs"]
mod tests;
