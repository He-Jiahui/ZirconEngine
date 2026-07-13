use crate::core::framework::render::{
    PostProcessVolumeExtract, RenderLayerSet, RenderViewExtract, VolumeComponentOverride,
    VolumeShapeExtract,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::components::{
    default_render_layer_mask, ColliderComponent, ColliderShape, PostProcessVolumeComponent,
};

use super::World;

pub(super) struct CollectedPostProcessVolumes {
    pub(super) extracts: Vec<PostProcessVolumeExtract>,
}

impl World {
    pub(super) fn collect_post_process_volumes_for_view(
        &self,
        view: &RenderViewExtract,
    ) -> CollectedPostProcessVolumes {
        self.collect_post_process_volumes(
            &post_process_volume_layers_for_view(view),
            view.camera.transform.translation,
        )
    }

    pub(super) fn collect_post_process_volumes(
        &self,
        camera_volume_layers: &RenderLayerSet,
        _camera_position: Vec3,
    ) -> CollectedPostProcessVolumes {
        let mut extracts = Vec::new();
        for (entity, volume) in self
            .post_process_volumes
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_volume_layers)
            })
            .filter(|(_, volume)| volume.active)
        {
            let volume_mask = RenderLayerSet::from_scene_schema_v1_mask(
                self.render_layer_mask(*entity)
                    .unwrap_or(default_render_layer_mask()),
            );
            if let Some(extract) =
                self.post_process_volume_extract(*entity, volume, volume_mask.clone())
            {
                extracts.push((*entity, extract));
            }
        }
        extracts.sort_by_key(|(entity, _)| *entity);
        CollectedPostProcessVolumes {
            extracts: extracts.into_iter().map(|(_, extract)| extract).collect(),
        }
    }

    fn post_process_volume_extract(
        &self,
        entity: crate::scene::EntityId,
        volume: &PostProcessVolumeComponent,
        volume_mask: RenderLayerSet,
    ) -> Option<PostProcessVolumeExtract> {
        let shape = self.post_process_volume_shape_extract(entity, volume)?;
        Some(PostProcessVolumeExtract::new(
            volume.active,
            shape,
            volume.priority,
            volume.weight,
            volume_mask,
            VolumeComponentOverride::from_profile(&volume.profile),
        ))
    }

    fn post_process_volume_shape_extract(
        &self,
        entity: crate::scene::EntityId,
        volume: &PostProcessVolumeComponent,
    ) -> Option<VolumeShapeExtract> {
        if volume.is_global {
            return Some(VolumeShapeExtract::global());
        }
        let collider = self.colliders.get(&entity)?;
        let world_transform = self.world_transform(entity).unwrap_or_default();
        volume_shape_extract_from_collider(world_transform, collider, volume.blend_distance)
    }
}

fn post_process_volume_layers_for_view(view: &RenderViewExtract) -> RenderLayerSet {
    let selected_camera = view.selected_camera_descriptor();
    let selected_layers = selected_camera
        .map(|camera| camera.volume_mask.clone())
        .unwrap_or_default();
    let selected_stack = selected_camera
        .map(|camera| camera.stack.as_slice())
        .unwrap_or(&[]);

    view.cameras
        .iter()
        .filter(|camera| {
            camera.entity == view.scene_camera_entity
                || camera
                    .entity
                    .is_some_and(|entity| selected_stack.contains(&entity))
        })
        .fold(selected_layers, |layers, camera| {
            layers.union(&camera.volume_mask)
        })
}

fn volume_shape_extract_from_collider(
    world_transform: Transform,
    collider: &ColliderComponent,
    blend_distance: Real,
) -> Option<VolumeShapeExtract> {
    let collider_transform = collider.local_transform;
    let center = world_transform.translation
        + world_transform.rotation * (world_transform.scale * collider_transform.translation);
    let rotation = world_transform.rotation * collider_transform.rotation;
    let scale = (world_transform.scale * collider_transform.scale).abs();

    match &collider.shape {
        ColliderShape::Box { half_extents } => Some(VolumeShapeExtract::box_shape(
            center,
            *half_extents * scale,
            rotation,
            blend_distance,
        )),
        ColliderShape::Sphere { radius } => Some(VolumeShapeExtract::sphere(
            center,
            *radius * max_component(scale),
            blend_distance,
        )),
        ColliderShape::Capsule { .. }
        | ColliderShape::Cylinder { .. }
        | ColliderShape::ConvexHull { .. }
        | ColliderShape::TriangleMesh { .. }
        | ColliderShape::HeightField { .. }
        | ColliderShape::Compound { .. } => None,
    }
}

fn max_component(value: Vec3) -> Real {
    value.x.max(value.y).max(value.z)
}
