use crate::core::framework::render::{
    FogVolumeData, PostProcessVolumeExtract, RenderLayerSet, RenderViewExtract,
    VolumeComponentOverride, VolumeShapeExtract, VOLUMETRIC_FOG_COMPONENT_ID,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::components::{
    default_render_layer_mask, ColliderComponent, ColliderShape, PostProcessVolumeComponent,
};

use super::World;

pub(super) struct CollectedPostProcessVolumes {
    pub(super) extracts: Vec<PostProcessVolumeExtract>,
    pub(super) fog_volumes: Vec<FogVolumeData>,
}

impl World {
    pub(super) fn collect_post_process_volumes_for_view(
        &self,
        view: &RenderViewExtract,
    ) -> CollectedPostProcessVolumes {
        self.collect_post_process_volumes(
            &post_process_volume_layers_for_view(view),
            &render_layers_for_view(view),
            view.camera.transform.translation,
        )
    }

    pub(super) fn collect_post_process_volumes(
        &self,
        camera_volume_layers: &RenderLayerSet,
        camera_render_layers: &RenderLayerSet,
        _camera_position: Vec3,
    ) -> CollectedPostProcessVolumes {
        let mut extracts = Vec::new();
        let mut fog_volumes = Vec::new();
        if let Some(component_id) = self.registered_component_id::<PostProcessVolumeComponent>() {
            self.archetype_index
                .for_each_table_component::<PostProcessVolumeComponent>(
                    component_id,
                    |entity, volume| {
                        if self.active_in_hierarchy(entity) != Some(true) || !volume.active {
                            return;
                        }
                        let volume_mask = RenderLayerSet::from_scene_schema_v1_mask(
                            self.render_layer_mask(entity)
                                .unwrap_or(default_render_layer_mask()),
                        );
                        let affects_post_process = volume_mask.intersects(camera_volume_layers);
                        let affects_local_fog = !volume.is_global
                            && volume.profile.volumetric_fog.is_some()
                            && volume_mask.intersects(camera_render_layers);
                        if !affects_post_process && !affects_local_fog {
                            return;
                        }
                        if let Some(extract) =
                            self.post_process_volume_extract(entity, volume, volume_mask.clone())
                        {
                            if affects_local_fog {
                                let settings = volume
                                    .profile
                                    .volumetric_fog
                                    .expect("local fog participation requires authored settings");
                                if let Some(fog_volume) = fog_volume_from_extract(
                                    entity,
                                    &extract,
                                    settings.density * extract.clamped_weight(),
                                    settings.albedo,
                                ) {
                                    fog_volumes.push((entity, fog_volume));
                                }
                            }
                            if affects_post_process {
                                extracts.push((entity, extract));
                            }
                        }
                    },
                );
        }
        // Filtering a priority-ordered source preserves priority order for every camera mask, so
        // per-camera volume evaluation only needs a linear influence scan in the common path.
        extracts.sort_by(|(left_entity, left), (right_entity, right)| {
            left.priority
                .partial_cmp(&right.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left_entity.cmp(right_entity))
        });
        fog_volumes.sort_by_key(|(entity, _)| *entity);
        CollectedPostProcessVolumes {
            extracts: extracts.into_iter().map(|(_, extract)| extract).collect(),
            fog_volumes: fog_volumes.into_iter().map(|(_, volume)| volume).collect(),
        }
    }

    fn post_process_volume_extract(
        &self,
        entity: crate::scene::EntityId,
        volume: &PostProcessVolumeComponent,
        volume_mask: RenderLayerSet,
    ) -> Option<PostProcessVolumeExtract> {
        let shape = self.post_process_volume_shape_extract(entity, volume)?;
        let mut overrides = VolumeComponentOverride::from_profile(&volume.profile);
        if !shape.is_global() {
            overrides.retain(|entry| entry.component_id != VOLUMETRIC_FOG_COMPONENT_ID);
        }
        Some(PostProcessVolumeExtract::new(
            volume.active,
            shape,
            volume.priority,
            volume.weight,
            volume_mask,
            overrides,
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
        let collider = self.get::<ColliderComponent>(entity)?;
        let world_transform = self.world_transform(entity).unwrap_or_default();
        volume_shape_extract_from_collider(world_transform, collider, volume.blend_distance)
    }
}

fn fog_volume_from_extract(
    entity: crate::scene::EntityId,
    extract: &PostProcessVolumeExtract,
    density: Real,
    albedo: Vec3,
) -> Option<FogVolumeData> {
    let (bounds_min, bounds_max) = match &extract.shape {
        VolumeShapeExtract::Global => return None,
        VolumeShapeExtract::Box {
            center,
            half_extents,
            rotation,
            ..
        } => {
            let aabb_half_extents = (*rotation * Vec3::X * half_extents.x).abs()
                + (*rotation * Vec3::Y * half_extents.y).abs()
                + (*rotation * Vec3::Z * half_extents.z).abs();
            (*center - aabb_half_extents, *center + aabb_half_extents)
        }
        VolumeShapeExtract::Sphere { center, radius, .. } => {
            let half_extents = Vec3::splat(*radius);
            (*center - half_extents, *center + half_extents)
        }
    };
    if !bounds_min.is_finite() || !bounds_max.is_finite() || !bounds_max.cmpgt(bounds_min).all() {
        return None;
    }
    Some(FogVolumeData {
        volume_id: entity,
        bounds_min,
        bounds_max,
        density: density.max(0.0),
        albedo: albedo.max(Vec3::ZERO),
        layer_mask: extract.volume_mask.clone(),
    })
}

fn render_layers_for_view(view: &RenderViewExtract) -> RenderLayerSet {
    let selected_camera = view.selected_camera_descriptor();
    let selected_layers = selected_camera
        .map(|camera| camera.culling_mask.clone())
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
            layers.union(&camera.culling_mask)
        })
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

#[cfg(test)]
mod tests {
    use super::render_layers_for_view;
    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderLayerSet, RenderViewExtract, ViewportCameraSnapshot,
    };

    #[test]
    fn render_volumetric_camera_stack_unions_overlay_culling_layers() {
        let mut base = camera_descriptor(10, 0b0001);
        base.stack = vec![20];
        let overlay = camera_descriptor(20, 0b0010);
        let unrelated = camera_descriptor(30, 0b0100);
        let view = RenderViewExtract::from_camera(base.camera.clone())
            .with_selected_camera_descriptor(base.clone())
            .with_cameras(vec![base, overlay, unrelated]);

        assert_eq!(
            render_layers_for_view(&view),
            RenderLayerSet::from_scene_schema_v1_mask(0b0011)
        );
    }

    fn camera_descriptor(entity: u64, culling_mask: u32) -> CameraRenderDescriptor {
        let mut camera = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(culling_mask);
        camera
    }
}
