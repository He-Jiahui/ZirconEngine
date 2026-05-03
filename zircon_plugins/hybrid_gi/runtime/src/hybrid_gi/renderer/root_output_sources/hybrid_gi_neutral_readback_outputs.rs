use std::collections::BTreeSet;

use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};
use zircon_runtime::core::framework::render::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
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
        let occupied_atlas_slots = snapshot.occupied_atlas_slots().to_vec();
        let occupied_capture_slots = snapshot.occupied_capture_slots().to_vec();
        let (atlas_samples, capture_samples) = snapshot.into_surface_cache_samples();

        Self {
            occupied_atlas_slots,
            occupied_capture_slots,
            atlas_samples: scene_prepare_samples(atlas_samples),
            capture_samples: scene_prepare_samples(capture_samples),
            voxel_clipmap_ids,
            voxel_samples,
            voxel_occupancy,
            voxel_occupancy_masks,
            voxel_cells,
            voxel_cell_samples,
            voxel_cell_dominant_nodes,
            voxel_cell_dominant_samples,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_outputs_project_hybrid_gi_gpu_readback() {
        let mut scene_prepare = HybridGiScenePrepareResourcesSnapshot::new(
            2,
            vec![9],
            vec![1, 2],
            vec![3],
            8,
            4,
            (64, 32),
            (16, 16),
            6,
        );
        scene_prepare.store_texture_slot_rgba_samples(
            vec![(1, [10, 20, 30, 255])],
            vec![(3, [40, 50, 60, 255])],
        );
        scene_prepare.store_voxel_resource_samples(
            vec![(9, [70, 80, 90, 255])],
            vec![(9, 0b1011)],
            vec![(9, 2, [100, 110, 120, 255])],
            vec![(9, 2, 4)],
            vec![(9, 2, 77)],
            vec![(9, 2, [130, 140, 150, 255])],
        );

        let readback = HybridGiGpuReadback::new(
            vec![(5, 7)],
            vec![11, 12],
            vec![21],
            vec![(11, [1, 2, 3]), (12, [4, 5, 6])],
            vec![(11, [7, 8, 9])],
            Some(scene_prepare),
        );

        let outputs = RenderHybridGiReadbackOutputs::from(readback);

        assert_eq!(
            outputs.cache_entries,
            vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }]
        );
        assert_eq!(outputs.completed_probe_ids, vec![11, 12]);
        assert_eq!(outputs.completed_trace_region_ids, vec![21]);
        assert_eq!(outputs.probe_irradiance_rgb, vec![[1, 2, 3], [4, 5, 6]]);
        assert_eq!(outputs.probe_rt_lighting_rgb, vec![[7, 8, 9]]);
        assert_eq!(outputs.scene_prepare.occupied_atlas_slots, vec![1, 2]);
        assert_eq!(outputs.scene_prepare.occupied_capture_slots, vec![3]);
        assert_eq!(
            outputs.scene_prepare.atlas_samples,
            vec![RenderHybridGiScenePrepareSample {
                index: 1,
                rgba8: [10, 20, 30, 255],
            }]
        );
        assert_eq!(
            outputs.scene_prepare.capture_samples,
            vec![RenderHybridGiScenePrepareSample {
                index: 3,
                rgba8: [40, 50, 60, 255],
            }]
        );
        assert_eq!(outputs.scene_prepare.voxel_clipmap_ids, vec![9]);
        assert_eq!(
            outputs.scene_prepare.voxel_samples,
            vec![RenderHybridGiScenePrepareSample {
                index: 9,
                rgba8: [70, 80, 90, 255],
            }]
        );
        assert_eq!(outputs.scene_prepare.voxel_occupancy, vec![3]);
        assert_eq!(
            outputs.scene_prepare.voxel_occupancy_masks,
            vec![RenderHybridGiVoxelOccupancyMaskRecord {
                clipmap_id: 9,
                occupancy_mask: 0b1011,
            }]
        );
        assert_eq!(
            outputs.scene_prepare.voxel_cells,
            vec![RenderHybridGiVoxelCellRecord {
                clipmap_id: 9,
                cell_id: 2,
                occupancy: 4,
            }]
        );
        assert_eq!(
            outputs.scene_prepare.voxel_cell_samples,
            vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 9,
                cell_id: 2,
                rgba8: [100, 110, 120, 255],
            }]
        );
        assert_eq!(
            outputs.scene_prepare.voxel_cell_dominant_nodes,
            vec![RenderHybridGiVoxelCellDominantNodeRecord {
                clipmap_id: 9,
                cell_id: 2,
                dominant_node_id: 77,
            }]
        );
        assert_eq!(
            outputs.scene_prepare.voxel_cell_dominant_samples,
            vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 9,
                cell_id: 2,
                rgba8: [130, 140, 150, 255],
            }]
        );
        assert_eq!(outputs.scene_prepare.texture_width, 64);
        assert_eq!(outputs.scene_prepare.texture_height, 32);
        assert_eq!(outputs.scene_prepare.texture_layers, 6);
    }

    #[test]
    fn neutral_outputs_stay_empty_without_hybrid_gi_gpu_readback_payload() {
        let outputs = RenderHybridGiReadbackOutputs::from(HybridGiGpuReadback::default());

        assert_eq!(outputs, RenderHybridGiReadbackOutputs::default());
    }
}
