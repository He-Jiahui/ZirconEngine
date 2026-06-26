use crate::core::framework::render::{
    RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot, RenderLayerSet,
    RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSpotLightSnapshot,
};
use crate::scene::components::default_render_layer_mask;

use super::super::World;

impl World {
    pub(super) fn collect_ambient_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderAmbientLightSnapshot> {
        let mut ambient_lights = self
            .ambient_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| {
                (
                    *entity,
                    RenderAmbientLightSnapshot {
                        color: light.color,
                        intensity: light.intensity,
                        renderer_degraded: false,
                        degradation_reason: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        ambient_lights.sort_by_key(|(entity, _)| *entity);
        ambient_lights.into_iter().map(|(_, light)| light).collect()
    }

    pub(super) fn collect_directional_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderDirectionalLightSnapshot> {
        let mut directional_lights = self
            .directional_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderDirectionalLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: RenderLayerSet::from_legacy_mask(
                    self.render_layer_mask(*entity)
                        .unwrap_or(default_render_layer_mask()),
                ),
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
                shadow: None,
            })
            .collect::<Vec<_>>();
        directional_lights.sort_by_key(|light| light.node_id);
        directional_lights
    }

    pub(super) fn collect_point_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderPointLightSnapshot> {
        let mut point_lights = self
            .point_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderPointLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: RenderLayerSet::from_legacy_mask(
                    self.render_layer_mask(*entity)
                        .unwrap_or(default_render_layer_mask()),
                ),
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                shadow: None,
            })
            .collect::<Vec<_>>();
        point_lights.sort_by_key(|light| light.node_id);
        point_lights
    }

    pub(super) fn collect_rect_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderRectLightSnapshot> {
        let mut rect_lights = self
            .rect_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| {
                let transform = self.world_transform(*entity).unwrap_or_default();
                RenderRectLightSnapshot {
                    node_id: *entity,
                    light_id: *entity,
                    layer_mask: RenderLayerSet::from_legacy_mask(
                        self.render_layer_mask(*entity)
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
                }
            })
            .collect::<Vec<_>>();
        rect_lights.sort_by_key(|light| light.node_id);
        rect_lights
    }

    pub(super) fn collect_spot_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderSpotLightSnapshot> {
        let mut spot_lights = self
            .spot_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderSpotLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: RenderLayerSet::from_legacy_mask(
                    self.render_layer_mask(*entity)
                        .unwrap_or(default_render_layer_mask()),
                ),
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                inner_angle_radians: light.inner_angle_radians,
                outer_angle_radians: light.outer_angle_radians,
                shadow: None,
            })
            .collect::<Vec<_>>();
        spot_lights.sort_by_key(|light| light.node_id);
        spot_lights
    }
}
