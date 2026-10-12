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
#[path = "tests/voxel_clipmap_debug.rs"]
mod tests;
