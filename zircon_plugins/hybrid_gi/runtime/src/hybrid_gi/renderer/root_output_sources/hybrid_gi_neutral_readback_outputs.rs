use std::collections::BTreeSet;

use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};
use zircon_runtime::core::framework::render::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiTraceTileRecord, RenderHybridGiVoxelCellDominantNodeRecord,
    RenderHybridGiVoxelCellRecord, RenderHybridGiVoxelCellSampleRecord,
    RenderHybridGiVoxelOccupancyMaskRecord,
};

impl From<HybridGiGpuReadback> for RenderHybridGiReadbackOutputs {
    fn from(readback: HybridGiGpuReadback) -> Self {
        let scene_prepare = readback
            .scene_prepare_resources()
            .map(RenderHybridGiScenePrepareReadbackOutputs::from)
            .unwrap_or_default();

        Self {
            cache_entries: readback
                .cache_entries()
                .iter()
                .map(|&(key, value)| RenderHybridGiCacheEntryRecord {
                    key: u64::from(key),
                    value: u64::from(value),
                })
                .collect(),
            completed_probe_ids: readback.completed_probe_ids().to_vec(),
            completed_trace_region_ids: readback.completed_trace_region_ids().to_vec(),
            probe_irradiance_rgb: rgb8_triplets_as_rgb16(readback.probe_irradiance_rgb()),
            probe_rt_lighting_rgb: rgb8_triplets_as_rgb16(readback.probe_trace_lighting_rgb()),
            radiance_cache_gpu_stage_dispatch_counts: *readback
                .radiance_cache_gpu_stage_dispatch_counts(),
            global_sdf_stats: None,
            scene_prepare,
        }
    }
}

impl From<HybridGiScenePrepareResourcesSnapshot> for RenderHybridGiScenePrepareReadbackOutputs {
    fn from(snapshot: HybridGiScenePrepareResourcesSnapshot) -> Self {
        let atlas_extent = snapshot.atlas_texture_extent();
        let texture_layers = snapshot.capture_layer_count();
        let voxel_clipmap_ids = neutral_voxel_clipmap_ids(&snapshot);
        let voxel_occupancy = neutral_voxel_occupancy_counts(&snapshot, &voxel_clipmap_ids);
        let voxel_occupancy_masks = neutral_voxel_occupancy_masks(&snapshot);
        let voxel_samples = snapshot
            .voxel_clipmap_rgba_samples()
            .iter()
            .map(|&(index, rgba8)| RenderHybridGiScenePrepareSample { index, rgba8 })
            .collect();
        let voxel_cells = snapshot
            .voxel_clipmap_cell_occupancy_counts()
            .iter()
            .map(
                |&(clipmap_id, cell_id, occupancy)| RenderHybridGiVoxelCellRecord {
                    clipmap_id,
                    cell_id,
                    occupancy,
                },
            )
            .collect();
        let voxel_cell_samples =
            neutral_voxel_cell_samples(snapshot.voxel_clipmap_cell_rgba_samples());
        let voxel_cell_dominant_nodes = neutral_voxel_cell_dominant_nodes(&snapshot);
        let voxel_cell_dominant_samples =
            neutral_voxel_cell_samples(snapshot.voxel_clipmap_cell_dominant_rgba_samples());
        let surface_cache_depth_samples =
            scene_prepare_samples(snapshot.surface_cache_depth_rgba_samples().to_vec());
        let probe_trace_tiles = neutral_probe_trace_tiles(snapshot.probe_trace_tiles());
        let probe_trace_diagnostics = snapshot.probe_trace_diagnostics().to_vec();
        let probe_trace_dispatch = snapshot.probe_trace_dispatch();
        let occupied_atlas_slots = snapshot.occupied_atlas_slots().to_vec();
        let occupied_capture_slots = snapshot.occupied_capture_slots().to_vec();
        let (atlas_samples, capture_samples) = snapshot.into_surface_cache_samples();

        Self {
            occupied_atlas_slots,
            occupied_capture_slots,
            atlas_samples: scene_prepare_samples(atlas_samples),
            capture_samples: scene_prepare_samples(capture_samples),
            surface_cache_depth_samples,
            // BUG: [CR-HYBRID-GI-0001] 此处恒空的页和 clipmap 列表会使后续场景追踪包缺少两类辐射来源。
            surface_cache_pages: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_clipmap_ids,
            voxel_samples,
            voxel_occupancy,
            voxel_occupancy_masks,
            voxel_cells,
            voxel_cell_samples,
            voxel_cell_dominant_nodes,
            voxel_cell_dominant_samples,
            probe_trace_tiles,
            probe_trace_diagnostics,
            probe_trace_dispatch,
            texture_width: atlas_extent.0,
            texture_height: atlas_extent.1,
            texture_layers,
        }
    }
}

fn rgb8_triplets_as_rgb16(samples: &[(u32, [u8; 3])]) -> Vec<[u16; 3]> {
    samples
        .iter()
        .map(|(_, rgb)| [u16::from(rgb[0]), u16::from(rgb[1]), u16::from(rgb[2])])
        .collect()
}

fn scene_prepare_samples(samples: Vec<(u32, [u8; 4])>) -> Vec<RenderHybridGiScenePrepareSample> {
    samples
        .into_iter()
        .map(|(index, rgba8)| RenderHybridGiScenePrepareSample { index, rgba8 })
        .collect()
}

fn neutral_voxel_clipmap_ids(snapshot: &HybridGiScenePrepareResourcesSnapshot) -> Vec<u32> {
    let mut ids: BTreeSet<u32> = snapshot.voxel_clipmap_ids().iter().copied().collect();
    ids.extend(
        snapshot
            .voxel_clipmap_occupancy_masks()
            .iter()
            .map(|&(clipmap_id, _)| clipmap_id),
    );
    ids.extend(
        snapshot
            .voxel_clipmap_cell_occupancy_counts()
            .iter()
            .map(|&(clipmap_id, _, _)| clipmap_id),
    );
    ids.into_iter().collect()
}

fn neutral_voxel_occupancy_counts(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    voxel_clipmap_ids: &[u32],
) -> Vec<u32> {
    voxel_clipmap_ids
        .iter()
        .map(|clipmap_id| {
            snapshot
                .voxel_clipmap_occupancy_masks()
                .iter()
                .find_map(|&(candidate_id, mask)| {
                    (candidate_id == *clipmap_id).then_some(mask.count_ones())
                })
                .unwrap_or_default()
        })
        .collect()
}

fn neutral_voxel_occupancy_masks(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
) -> Vec<RenderHybridGiVoxelOccupancyMaskRecord> {
    snapshot
        .voxel_clipmap_occupancy_masks()
        .iter()
        .map(
            |&(clipmap_id, occupancy_mask)| RenderHybridGiVoxelOccupancyMaskRecord {
                clipmap_id,
                occupancy_mask,
            },
        )
        .collect()
}

fn neutral_voxel_cell_samples(
    samples: &[(u32, u32, [u8; 4])],
) -> Vec<RenderHybridGiVoxelCellSampleRecord> {
    samples
        .iter()
        .map(
            |&(clipmap_id, cell_id, rgba8)| RenderHybridGiVoxelCellSampleRecord {
                clipmap_id,
                cell_id,
                rgba8,
            },
        )
        .collect()
}

fn neutral_voxel_cell_dominant_nodes(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
) -> Vec<RenderHybridGiVoxelCellDominantNodeRecord> {
    snapshot
        .voxel_clipmap_cell_dominant_node_ids()
        .iter()
        .map(
            |&(clipmap_id, cell_id, dominant_node_id)| RenderHybridGiVoxelCellDominantNodeRecord {
                clipmap_id,
                cell_id,
                dominant_node_id,
            },
        )
        .collect()
}

fn neutral_probe_trace_tiles(tiles: &[(u32, u32, u32, u32)]) -> Vec<RenderHybridGiTraceTileRecord> {
    tiles
        .iter()
        .map(
            |&(tile_id, probe_id, trace_region_id, ray_count)| RenderHybridGiTraceTileRecord {
                tile_id,
                probe_id,
                trace_region_id,
                ray_count,
            },
        )
        .collect()
}

#[cfg(test)]
#[path = "tests/hybrid_gi_neutral_readback_outputs.rs"]
mod tests;
