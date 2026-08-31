use crate::hybrid_gi::types::{
    hybrid_gi_voxel_clipmap_aabb_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::math::Vec3;

use super::card_capture_shading::{mesh_capture_radiance, rgba8_from_color_with_alpha};
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::material_capture_source::HybridGiMaterialCaptureSource;

pub(super) struct SceneVoxelClipmapCellSamples {
    pub(super) rgba_samples: Vec<(u32, [u8; 4])>,
    pub(super) dominant_node_ids: Vec<(u32, u64)>,
    pub(super) dominant_rgba_samples: Vec<(u32, [u8; 4])>,
}

fn mesh_world_bounds(
    inputs: &HybridGiPrepareExecutionInputs,
    mesh: &RenderMeshSnapshot,
) -> Option<RenderMeshBounds> {
    inputs
        .scene_mesh_world_bounds
        .binary_search_by_key(&mesh.stable_instance_key, |(stable_instance_key, _)| {
            *stable_instance_key
        })
        .ok()
        .map(|index| inputs.scene_mesh_world_bounds[index].1)
}

fn mesh_cell_ranges(
    clipmap: &HybridGiPrepareVoxelClipmap,
    mesh: &RenderMeshSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Option<[(usize, usize); 3]> {
    let bounds = mesh_world_bounds(inputs, mesh)?;
    hybrid_gi_voxel_clipmap_aabb_cell_ranges(
        clipmap,
        Vec3::from_array(bounds.min),
        Vec3::from_array(bounds.max),
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn scene_voxel_clipmap_occupancy_mask(
    clipmap: &HybridGiPrepareVoxelClipmap,
    inputs: &HybridGiPrepareExecutionInputs,
) -> u64 {
    debug_assert!(HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT <= u64::BITS as usize);

    let mut occupancy_mask = 0_u64;
    for mesh in inputs.scene_meshes.iter() {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh, inputs)
        else {
            continue;
        };

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    occupancy_mask |= 1_u64 << hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                }
            }
        }
    }

    occupancy_mask
}

#[cfg(test)]
pub(super) fn scene_voxel_clipmap_cell_rgba_samples(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    debug_assert!(HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT <= u64::BITS as usize);

    let mut cell_radiance = [Vec3::ZERO; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut cell_has_sample = [false; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    for mesh in inputs.scene_meshes.iter() {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh, inputs)
        else {
            continue;
        };

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                    cell_has_sample[cell_index] = true;
                    cell_radiance[cell_index] += mesh_capture_radiance(
                        mesh,
                        hybrid_gi_voxel_clipmap_cell_center(clipmap, x, y, z),
                        streamer,
                        inputs,
                    );
                }
            }
        }
    }

    cell_radiance
        .into_iter()
        .zip(cell_has_sample)
        .enumerate()
        .map(|(cell_index, (radiance, has_sample))| {
            (
                cell_index as u32,
                rgba8_from_color_with_alpha(radiance, if has_sample { 255 } else { 0 }),
            )
        })
        .collect()
}

pub(super) fn scene_voxel_clipmap_cell_samples(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> SceneVoxelClipmapCellSamples {
    debug_assert!(HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT <= u64::BITS as usize);

    let mut cell_radiance = [Vec3::ZERO; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut cell_has_sample = [false; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_node_ids = [0_u64; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_strengths = [0.0_f32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_rgba_samples = [[0, 0, 0, 0]; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut clipmap_has_scene_sample = false;

    for mesh in inputs.scene_meshes.iter() {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh, inputs)
        else {
            continue;
        };
        clipmap_has_scene_sample = true;

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                    let radiance = mesh_capture_radiance(
                        mesh,
                        hybrid_gi_voxel_clipmap_cell_center(clipmap, x, y, z),
                        streamer,
                        inputs,
                    );
                    cell_has_sample[cell_index] = true;
                    cell_radiance[cell_index] += radiance;
                    let strength = radiance.x + radiance.y + radiance.z;
                    let should_replace = dominant_node_ids[cell_index] == 0
                        || strength > dominant_strengths[cell_index]
                        || (strength == dominant_strengths[cell_index]
                            && mesh.node_id > dominant_node_ids[cell_index]);
                    if should_replace {
                        dominant_node_ids[cell_index] = mesh.node_id;
                        dominant_strengths[cell_index] = strength;
                        dominant_rgba_samples[cell_index] =
                            rgba8_from_color_with_alpha(radiance, 255);
                    }
                }
            }
        }
    }

    let dominant_node_id_samples = dominant_node_ids
        .iter()
        .copied()
        .enumerate()
        .map(|(cell_index, node_id)| (cell_index as u32, node_id))
        .collect();
    let dominant_rgba_samples = dominant_node_ids
        .into_iter()
        .zip(dominant_rgba_samples)
        .enumerate()
        .map(|(cell_index, (node_id, rgba))| {
            let rgba = if clipmap_has_scene_sample && node_id == 0 {
                [0, 0, 0, 255]
            } else {
                rgba
            };
            (cell_index as u32, rgba)
        })
        .collect();

    SceneVoxelClipmapCellSamples {
        rgba_samples: cell_radiance
            .into_iter()
            .zip(cell_has_sample)
            .enumerate()
            .map(|(cell_index, (radiance, has_sample))| {
                (
                    cell_index as u32,
                    rgba8_from_color_with_alpha(radiance, if has_sample { 255 } else { 0 }),
                )
            })
            .collect(),
        dominant_node_ids: dominant_node_id_samples,
        dominant_rgba_samples,
    }
}

#[cfg(test)]
pub(super) fn scene_voxel_clipmap_cell_dominant_node_ids(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, u64)> {
    scene_voxel_clipmap_cell_dominant_entries(clipmap, streamer, inputs)
        .into_iter()
        .map(|(cell_index, node_id, _)| (cell_index, node_id))
        .collect()
}

#[cfg(test)]
pub(super) fn scene_voxel_clipmap_cell_dominant_rgba_samples(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    scene_voxel_clipmap_cell_dominant_entries(clipmap, streamer, inputs)
        .into_iter()
        .map(|(cell_index, _, rgba)| (cell_index, rgba))
        .collect()
}

#[cfg(test)]
fn scene_voxel_clipmap_cell_dominant_entries(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, u64, [u8; 4])> {
    let mut dominant_node_ids = [0_u64; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_strengths = [0.0_f32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_rgba_samples = [[0, 0, 0, 0]; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut clipmap_has_scene_sample = false;

    for mesh in inputs.scene_meshes.iter() {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh, inputs)
        else {
            continue;
        };
        clipmap_has_scene_sample = true;

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                    let radiance = mesh_capture_radiance(
                        mesh,
                        hybrid_gi_voxel_clipmap_cell_center(clipmap, x, y, z),
                        streamer,
                        inputs,
                    );
                    let strength = radiance.x + radiance.y + radiance.z;
                    let should_replace = dominant_node_ids[cell_index] == 0
                        || strength > dominant_strengths[cell_index]
                        || (strength == dominant_strengths[cell_index]
                            && mesh.node_id > dominant_node_ids[cell_index]);
                    if should_replace {
                        dominant_node_ids[cell_index] = mesh.node_id;
                        dominant_strengths[cell_index] = strength;
                        dominant_rgba_samples[cell_index] =
                            rgba8_from_color_with_alpha(radiance, 255);
                    }
                }
            }
        }
    }

    dominant_node_ids
        .into_iter()
        .zip(dominant_rgba_samples)
        .enumerate()
        .map(|(cell_index, (node_id, rgba))| {
            let rgba = if clipmap_has_scene_sample && node_id == 0 {
                [0, 0, 0, 255]
            } else {
                rgba
            };
            (cell_index as u32, node_id, rgba)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::material_capture_source::{
        HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureTextureKey,
    };
    use zircon_runtime::core::framework::render::{
        RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
        RenderSpotLightSnapshot,
    };
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::math::{Transform, Vec3, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    use super::*;

    struct EmptyMaterialCaptureSource;

    impl HybridGiMaterialCaptureSource for EmptyMaterialCaptureSource {
        fn material_capture_seed(&self, _id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
            None
        }

        fn sample_texture_rgba(
            &self,
            _texture: Option<HybridGiMaterialCaptureTextureKey>,
            _uv: [f32; 2],
        ) -> Option<Vec4> {
            None
        }
    }

    #[test]
    fn scene_voxel_clipmap_occupancy_mask_moves_when_mesh_crosses_cells() {
        let clipmap = HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 8.0,
        };
        let left = inputs_with_mesh(clipmap.clone(), Vec3::new(-3.0, 0.0, 0.0));
        let right = inputs_with_mesh(clipmap.clone(), Vec3::new(3.0, 0.0, 0.0));

        let left_mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &left);
        let right_mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &right);

        assert_ne!(left_mask, right_mask);
    }

    fn inputs_with_mesh(
        clipmap: HybridGiPrepareVoxelClipmap,
        translation: Vec3,
    ) -> HybridGiPrepareExecutionInputs {
        HybridGiPrepareExecutionInputs {
            cache_entries: Vec::new(),
            resident_probe_inputs: Vec::new(),
            pending_probe_inputs: Vec::new(),
            radiance_cache_update_inputs: Vec::new(),
            radiance_cache_consume_inputs: Vec::new(),
            trace_region_inputs: Vec::new(),
            scene_card_capture_requests: Vec::new(),
            scene_surface_cache_depth_source_samples: Vec::new(),
            scene_surface_cache_page_contents: Vec::new(),
            scene_card_capture_descriptor_count: 0,
            scene_voxel_clipmaps: vec![clipmap],
            scene_voxel_cells: Vec::new(),
            scene_mesh_world_bounds: vec![(
                render_mesh_stable_instance_key(11, 0),
                RenderMeshBounds::from_min_max(
                    (translation - Vec3::splat(0.5)).to_array(),
                    (translation + Vec3::splat(0.5)).to_array(),
                ),
            )]
            .into(),
            scene_meshes: vec![mesh_at(translation)].into(),
            directional_lights: Vec::<RenderDirectionalLightSnapshot>::new(),
            point_lights: Vec::<RenderPointLightSnapshot>::new(),
            spot_lights: Vec::<RenderSpotLightSnapshot>::new(),
            cache_word_count: 0,
            completed_probe_word_count: 0,
            completed_trace_word_count: 0,
            irradiance_word_count: 0,
            trace_lighting_word_count: 0,
            trace_diagnostic_word_count: 0,
        }
    }

    #[test]
    fn elongated_mesh_bounds_only_cover_intersecting_cell_rows() {
        let clipmap = HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 8.0,
        };
        let mut inputs = inputs_with_mesh(clipmap.clone(), Vec3::ZERO);
        Arc::make_mut(&mut inputs.scene_mesh_world_bounds)[0].1 =
            RenderMeshBounds::from_min_max([-7.5, -0.25, -0.25], [7.5, 0.25, 0.25]);

        let mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &inputs);

        assert_eq!(mask.count_ones(), 16);
    }

    #[test]
    fn missing_authoritative_bounds_do_not_project_voxel_cell_samples() {
        let clipmap = HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 8.0,
        };
        let mut inputs = inputs_with_mesh(clipmap.clone(), Vec3::ZERO);
        inputs.scene_mesh_world_bounds = Arc::from([]);
        let streamer = EmptyMaterialCaptureSource;

        let samples = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

        assert!(samples
            .rgba_samples
            .iter()
            .all(|(_, rgba)| *rgba == [0, 0, 0, 0]));
        assert!(samples
            .dominant_node_ids
            .iter()
            .all(|(_, node_id)| *node_id == 0));
        assert!(samples
            .dominant_rgba_samples
            .iter()
            .all(|(_, rgba)| *rgba == [0, 0, 0, 0]));
    }

    #[test]
    fn aggregated_voxel_cell_samples_preserve_all_legacy_projections() {
        let clipmap = HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 8.0,
        };
        let inputs = inputs_with_mesh(clipmap.clone(), Vec3::new(1.5, 0.0, 0.0));
        let streamer = EmptyMaterialCaptureSource;

        let aggregated = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

        assert_eq!(
            aggregated.rgba_samples,
            scene_voxel_clipmap_cell_rgba_samples(&clipmap, &streamer, &inputs)
        );
        assert_eq!(
            aggregated.dominant_node_ids,
            scene_voxel_clipmap_cell_dominant_node_ids(&clipmap, &streamer, &inputs)
        );
        assert_eq!(
            aggregated.dominant_rgba_samples,
            scene_voxel_clipmap_cell_dominant_rgba_samples(&clipmap, &streamer, &inputs)
        );
    }

    #[test]
    fn aggregated_voxel_cell_samples_keep_the_larger_node_id_on_equal_strength() {
        let clipmap = HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 8.0,
        };
        let translation = Vec3::ZERO;
        let mut inputs = inputs_with_mesh(clipmap.clone(), translation);
        let higher_node_id = 19;
        let mut meshes = inputs.scene_meshes.to_vec();
        meshes.push(mesh_at_node(higher_node_id, translation));
        inputs.scene_meshes = meshes.into();
        let mut bounds = inputs.scene_mesh_world_bounds.to_vec();
        bounds.push((
            render_mesh_stable_instance_key(higher_node_id, 0),
            RenderMeshBounds::from_min_max(
                (translation - Vec3::splat(0.5)).to_array(),
                (translation + Vec3::splat(0.5)).to_array(),
            ),
        ));
        bounds.sort_unstable_by_key(|(stable_instance_key, _)| *stable_instance_key);
        inputs.scene_mesh_world_bounds = bounds.into();
        let streamer = EmptyMaterialCaptureSource;

        let samples = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

        assert!(samples
            .dominant_node_ids
            .iter()
            .filter(|(_, node_id)| *node_id != 0)
            .all(|(_, node_id)| *node_id == higher_node_id));
    }

    fn mesh_at(translation: Vec3) -> RenderMeshSnapshot {
        mesh_at_node(11, translation)
    }

    fn mesh_at_node(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
        let transform = Transform::from_translation(translation);
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
            transform_revision: render_mesh_transform_revision(&transform),
            transform,
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "builtin://cube",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "builtin://material/default",
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            static_state: RenderMeshStaticState::from_transform_static(true),
            common: RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                is_static: true,
                ..RendererCommon::default()
            },
        }
    }
}
