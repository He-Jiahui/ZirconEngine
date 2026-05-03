use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiGpuCompletion {
    cache_entries: Vec<(u32, u32)>,
    completed_probe_ids: Vec<u32>,
    completed_trace_region_ids: Vec<u32>,
    probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    scene_prepare: Option<RenderHybridGiScenePrepareReadbackOutputs>,
}

impl HybridGiGpuCompletion {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cache_entries: Vec<(u32, u32)>,
        completed_probe_ids: Vec<u32>,
        completed_trace_region_ids: Vec<u32>,
        probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
        probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
        scene_prepare: Option<RenderHybridGiScenePrepareReadbackOutputs>,
    ) -> Self {
        Self {
            cache_entries,
            completed_probe_ids,
            completed_trace_region_ids,
            probe_irradiance_rgb,
            probe_trace_lighting_rgb,
            scene_prepare,
        }
    }

    pub fn cache_entries(&self) -> &[(u32, u32)] {
        &self.cache_entries
    }

    pub fn completed_probe_ids(&self) -> &[u32] {
        &self.completed_probe_ids
    }

    pub fn completed_trace_region_ids(&self) -> &[u32] {
        &self.completed_trace_region_ids
    }

    pub fn probe_irradiance_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_irradiance_rgb
    }

    pub fn probe_trace_lighting_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_trace_lighting_rgb
    }

    pub fn scene_prepare(&self) -> Option<&RenderHybridGiScenePrepareReadbackOutputs> {
        self.scene_prepare.as_ref()
    }

    pub(crate) fn from_readback_outputs(outputs: RenderHybridGiReadbackOutputs) -> Option<Self> {
        let completed_probe_ids = outputs.completed_probe_ids;
        let cache_entries: Vec<(u32, u32)> = outputs
            .cache_entries
            .into_iter()
            .filter_map(|entry| -> Option<(u32, u32)> {
                Some((
                    u32::try_from(entry.key).ok()?,
                    u32::try_from(entry.value).ok()?,
                ))
            })
            .collect();
        let probe_irradiance_rgb =
            probe_colors_from_neutral_outputs(&completed_probe_ids, outputs.probe_irradiance_rgb);
        let probe_trace_lighting_rgb =
            probe_colors_from_neutral_outputs(&completed_probe_ids, outputs.probe_rt_lighting_rgb);
        let scene_prepare = outputs
            .scene_prepare
            .has_runtime_feedback_payload()
            .then_some(outputs.scene_prepare);

        if cache_entries.is_empty()
            && completed_probe_ids.is_empty()
            && outputs.completed_trace_region_ids.is_empty()
            && probe_irradiance_rgb.is_empty()
            && probe_trace_lighting_rgb.is_empty()
            && scene_prepare.is_none()
        {
            return None;
        }

        Some(Self::new(
            cache_entries,
            completed_probe_ids,
            outputs.completed_trace_region_ids,
            probe_irradiance_rgb,
            probe_trace_lighting_rgb,
            scene_prepare,
        ))
    }
}

fn probe_colors_from_neutral_outputs(
    probe_ids: &[u32],
    colors: Vec<[u16; 3]>,
) -> Vec<(u32, [u8; 3])> {
    probe_ids
        .iter()
        .copied()
        .zip(colors)
        .map(|(probe_id, rgb)| {
            (
                probe_id,
                [
                    rgb[0].min(u16::from(u8::MAX)) as u8,
                    rgb[1].min(u16::from(u8::MAX)) as u8,
                    rgb[2].min(u16::from(u8::MAX)) as u8,
                ],
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
        RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
        RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
    };

    #[test]
    fn gpu_completion_projects_neutral_hybrid_gi_readback_outputs() {
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                cache_entries: vec![RenderHybridGiCacheEntryRecord { key: 17, value: 3 }],
                completed_probe_ids: vec![17, 19],
                completed_trace_region_ids: vec![5],
                probe_irradiance_rgb: vec![[16, 260, 64], [4, 8, 12]],
                probe_rt_lighting_rgb: vec![[1, 2, 3]],
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    atlas_samples: vec![RenderHybridGiScenePrepareSample {
                        index: 9,
                        rgba8: [1, 2, 3, 4],
                    }],
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
            })
            .expect("nonempty readback should create completion");

        assert_eq!(completion.cache_entries(), &[(17, 3)]);
        assert_eq!(completion.completed_probe_ids(), &[17, 19]);
        assert_eq!(completion.completed_trace_region_ids(), &[5]);
        assert_eq!(
            completion.probe_irradiance_rgb(),
            &[(17, [16, 255, 64]), (19, [4, 8, 12])]
        );
        assert_eq!(completion.probe_trace_lighting_rgb(), &[(17, [1, 2, 3])]);
        assert_eq!(
            completion.scene_prepare().unwrap().atlas_samples[0].rgba8,
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn gpu_completion_skips_empty_neutral_hybrid_gi_readback_outputs() {
        assert!(HybridGiGpuCompletion::from_readback_outputs(
            RenderHybridGiReadbackOutputs::default()
        )
        .is_none());
    }

    #[test]
    fn gpu_completion_skips_non_runtime_consumable_scene_prepare_metadata() {
        assert!(
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    occupied_atlas_slots: vec![3],
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            })
            .is_none()
        );
    }

    #[test]
    fn gpu_completion_keeps_voxel_scene_prepare_readback_payload() {
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    voxel_occupancy_masks: vec![RenderHybridGiVoxelOccupancyMaskRecord {
                        clipmap_id: 4,
                        occupancy_mask: 0b1001,
                    }],
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            })
            .expect("voxel readback payload should keep a completion");

        assert_eq!(
            completion.scene_prepare().unwrap().voxel_occupancy_masks[0].occupancy_mask,
            0b1001
        );
    }

    #[test]
    fn gpu_completion_keeps_voxel_cell_scene_prepare_readback_payload() {
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    voxel_cell_dominant_samples: vec![RenderHybridGiVoxelCellSampleRecord {
                        clipmap_id: 4,
                        cell_id: 9,
                        rgba8: [32, 48, 64, 255],
                    }],
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            })
            .expect("voxel cell readback payload should keep a completion");

        assert_eq!(
            completion
                .scene_prepare()
                .unwrap()
                .voxel_cell_dominant_samples[0]
                .rgba8,
            [32, 48, 64, 255]
        );
    }
}
