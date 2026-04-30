use std::collections::BTreeMap;

use super::super::card_capture_shading::scene_voxel_clipmap_rgba;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::super::voxel_clipmap_debug::{
    scene_voxel_clipmap_cell_dominant_node_ids, scene_voxel_clipmap_cell_dominant_rgba_samples,
    scene_voxel_clipmap_cell_rgba_samples,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
};

pub(super) fn store_scene_prepare_voxel_resource_samples(
    snapshot: &mut HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    let voxel_cell_occupancy_counts = voxel_cell_occupancy_counts_from_scene_prepare(
        &inputs.scene_voxel_clipmaps,
        &inputs.scene_voxel_cells,
    );
    let runtime_voxel_clipmap_radiance_overrides =
        runtime_voxel_clipmap_radiance_rgba_overrides(&inputs.scene_voxel_cells);
    let runtime_voxel_cell_radiance_overrides =
        runtime_voxel_cell_radiance_rgba_overrides(&inputs.scene_voxel_cells);
    let runtime_voxel_cell_dominant_node_overrides =
        runtime_voxel_cell_dominant_node_overrides(&inputs.scene_voxel_cells);
    let voxel_clipmap_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .map(|clipmap| {
            (
                clipmap.clipmap_id,
                runtime_voxel_clipmap_radiance_overrides
                    .get(&clipmap.clipmap_id)
                    .copied()
                    .unwrap_or_else(|| scene_voxel_clipmap_rgba(clipmap, streamer, inputs)),
            )
        })
        .collect();
    let voxel_clipmap_occupancy_masks = voxel_occupancy_masks_from_counts(
        &inputs.scene_voxel_clipmaps,
        &voxel_cell_occupancy_counts,
    );
    let voxel_clipmap_cell_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_rgba_samples(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, rgba)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_radiance_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(rgba),
                    )
                })
        })
        .collect();
    let voxel_clipmap_cell_dominant_node_ids = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_dominant_node_ids(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, node_id)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_dominant_node_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(node_id),
                    )
                })
        })
        .collect();
    let voxel_clipmap_cell_dominant_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_dominant_rgba_samples(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, rgba)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_radiance_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(rgba),
                    )
                })
        })
        .collect();
    snapshot.store_voxel_resource_samples(
        voxel_clipmap_rgba_samples,
        voxel_clipmap_occupancy_masks,
        voxel_clipmap_cell_rgba_samples,
        voxel_cell_occupancy_counts,
        voxel_clipmap_cell_dominant_node_ids,
        voxel_clipmap_cell_dominant_rgba_samples,
    );
}

fn voxel_cell_occupancy_counts_from_scene_prepare(
    clipmaps: &[HybridGiPrepareVoxelClipmap],
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> Vec<(u32, u32, u32)> {
    let occupancy_by_cell = voxel_cells
        .iter()
        .map(|cell| ((cell.clipmap_id, cell.cell_index), cell.occupancy_count))
        .collect::<BTreeMap<_, _>>();
    let occupancy_by_cell = &occupancy_by_cell;

    clipmaps
        .iter()
        .flat_map(|clipmap| {
            (0..HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT).map(|cell_index| {
                (
                    clipmap.clipmap_id,
                    cell_index as u32,
                    occupancy_by_cell
                        .get(&(clipmap.clipmap_id, cell_index as u32))
                        .copied()
                        .unwrap_or(0),
                )
            })
        })
        .collect()
}

fn runtime_voxel_cell_radiance_rgba_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<(u32, u32), [u8; 4]> {
    voxel_cells
        .iter()
        .filter(|cell| cell.occupancy_count > 0)
        .filter_map(|cell| {
            cell.radiance_present.then_some((
                (cell.clipmap_id, cell.cell_index),
                [
                    cell.radiance_rgb[0],
                    cell.radiance_rgb[1],
                    cell.radiance_rgb[2],
                    255,
                ],
            ))
        })
        .collect()
}

fn runtime_voxel_clipmap_radiance_rgba_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<u32, [u8; 4]> {
    let mut weighted_rgb_sums = BTreeMap::<u32, [u64; 3]>::new();
    let mut total_weights = BTreeMap::<u32, u64>::new();

    for cell in voxel_cells.iter().filter(|cell| cell.occupancy_count > 0) {
        if !cell.radiance_present {
            continue;
        }

        let weight = cell.occupancy_count.max(1) as u64;
        let rgb_sum = weighted_rgb_sums
            .entry(cell.clipmap_id)
            .or_insert([0, 0, 0]);
        rgb_sum[0] += cell.radiance_rgb[0] as u64 * weight;
        rgb_sum[1] += cell.radiance_rgb[1] as u64 * weight;
        rgb_sum[2] += cell.radiance_rgb[2] as u64 * weight;
        *total_weights.entry(cell.clipmap_id).or_insert(0) += weight;
    }

    weighted_rgb_sums
        .into_iter()
        .filter_map(|(clipmap_id, rgb_sum)| {
            let total_weight = total_weights.get(&clipmap_id).copied().unwrap_or(0);
            (total_weight > 0).then_some((
                clipmap_id,
                [
                    ((rgb_sum[0] + total_weight / 2) / total_weight) as u8,
                    ((rgb_sum[1] + total_weight / 2) / total_weight) as u8,
                    ((rgb_sum[2] + total_weight / 2) / total_weight) as u8,
                    255,
                ],
            ))
        })
        .collect()
}

fn runtime_voxel_cell_dominant_node_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<(u32, u32), u64> {
    voxel_cells
        .iter()
        .filter(|cell| cell.occupancy_count > 0)
        .filter_map(|cell| {
            (cell.dominant_card_id != 0).then_some((
                (cell.clipmap_id, cell.cell_index),
                cell.dominant_card_id as u64,
            ))
        })
        .collect()
}

fn voxel_occupancy_masks_from_counts(
    clipmaps: &[HybridGiPrepareVoxelClipmap],
    counts: &[(u32, u32, u32)],
) -> Vec<(u32, u64)> {
    let mut masks_by_clipmap = clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, 0_u64))
        .collect::<BTreeMap<_, _>>();
    for &(clipmap_id, cell_index, occupancy_count) in counts {
        if occupancy_count == 0 || cell_index >= u64::BITS {
            continue;
        }
        *masks_by_clipmap.entry(clipmap_id).or_default() |= 1_u64 << cell_index;
    }
    masks_by_clipmap.into_iter().collect()
}
