use crate::core::framework::render::RenderMeshSnapshot;
use crate::core::math::Vec3;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_bounds_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
};

use super::card_capture_shading::{mesh_capture_radiance, rgba8_from_color_with_alpha};
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

const MIN_MESH_BOUNDS_RADIUS: f32 = 0.5;

fn mesh_bounds_radius(mesh: &RenderMeshSnapshot) -> f32 {
    (mesh.transform.scale.abs().max_element() * 0.5).max(MIN_MESH_BOUNDS_RADIUS)
}

fn mesh_cell_ranges(
    clipmap: &HybridGiPrepareVoxelClipmap,
    mesh: &RenderMeshSnapshot,
) -> Option<[(usize, usize); 3]> {
    hybrid_gi_voxel_clipmap_bounds_cell_ranges(
        clipmap,
        mesh.transform.translation,
        mesh_bounds_radius(mesh),
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn scene_voxel_clipmap_occupancy_mask(
    clipmap: &HybridGiPrepareVoxelClipmap,
    inputs: &HybridGiPrepareExecutionInputs,
) -> u64 {
    debug_assert!(HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT <= u64::BITS as usize);

    let mut occupancy_mask = 0_u64;
    for mesh in &inputs.scene_meshes {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh)
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

pub(super) fn scene_voxel_clipmap_cell_rgba_samples(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    debug_assert!(HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT <= u64::BITS as usize);

    let mut cell_radiance = [Vec3::ZERO; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut cell_has_sample = [false; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    for mesh in &inputs.scene_meshes {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh)
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

pub(super) fn scene_voxel_clipmap_cell_dominant_node_ids(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, u64)> {
    scene_voxel_clipmap_cell_dominant_entries(clipmap, streamer, inputs)
        .into_iter()
        .map(|(cell_index, node_id, _)| (cell_index, node_id))
        .collect()
}

pub(super) fn scene_voxel_clipmap_cell_dominant_rgba_samples(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    scene_voxel_clipmap_cell_dominant_entries(clipmap, streamer, inputs)
        .into_iter()
        .map(|(cell_index, _, rgba)| (cell_index, rgba))
        .collect()
}

fn scene_voxel_clipmap_cell_dominant_entries(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, u64, [u8; 4])> {
    let mut dominant_node_ids = [0_u64; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_strengths = [0.0_f32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut dominant_rgba_samples = [[0, 0, 0, 0]; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT];
    let mut clipmap_has_scene_sample = false;

    for mesh in &inputs.scene_meshes {
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            mesh_cell_ranges(clipmap, mesh)
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
    use crate::core::framework::render::{
        RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
        RenderSpotLightSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, Vec3, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    use super::*;

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
            trace_region_inputs: Vec::new(),
            scene_card_capture_requests: Vec::new(),
            scene_voxel_clipmaps: vec![clipmap],
            scene_voxel_cells: Vec::new(),
            scene_meshes: vec![mesh_at(translation)],
            directional_lights: Vec::<RenderDirectionalLightSnapshot>::new(),
            point_lights: Vec::<RenderPointLightSnapshot>::new(),
            spot_lights: Vec::<RenderSpotLightSnapshot>::new(),
            cache_word_count: 0,
            completed_probe_word_count: 0,
            completed_trace_word_count: 0,
            irradiance_word_count: 0,
            trace_lighting_word_count: 0,
        }
    }

    fn mesh_at(translation: Vec3) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "builtin://cube",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "builtin://material/default",
            )),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            render_layer_mask: u32::MAX,
        }
    }
}
