use crate::core::framework::render::{
    RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiGpuCompletion {
    cache_entries: Vec<(u32, u32)>,
    completed_probe_ids: Vec<u32>,
    completed_trace_region_ids: Vec<u32>,
    probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    global_sdf_stats: Option<RenderHybridGiGlobalSdfStats>,
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
            radiance_cache_gpu_stage_dispatch_counts: Default::default(),
            global_sdf_stats: None,
            scene_prepare,
        }
    }

    pub fn with_radiance_cache_gpu_stage_dispatch_counts(
        mut self,
        counts: [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    ) -> Self {
        self.radiance_cache_gpu_stage_dispatch_counts = counts;
        self
    }

    pub fn with_global_sdf_stats(mut self, stats: Option<RenderHybridGiGlobalSdfStats>) -> Self {
        self.global_sdf_stats = stats;
        self
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

    pub fn radiance_cache_gpu_stage_dispatch_counts(
        &self,
    ) -> [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT] {
        self.radiance_cache_gpu_stage_dispatch_counts
    }

    pub fn global_sdf_stats(&self) -> Option<RenderHybridGiGlobalSdfStats> {
        self.global_sdf_stats
    }

    pub fn scene_prepare(&self) -> Option<&RenderHybridGiScenePrepareReadbackOutputs> {
        self.scene_prepare.as_ref()
    }

    pub(crate) fn from_readback_outputs(outputs: RenderHybridGiReadbackOutputs) -> Option<Self> {
        let radiance_cache_gpu_stage_dispatch_counts =
            outputs.radiance_cache_gpu_stage_dispatch_counts;
        let global_sdf_stats = outputs.global_sdf_stats;
        let completed_probe_ids = outputs.completed_probe_ids;
        let cache_entry_records = outputs.cache_entries;
        let mut cache_entries = Vec::with_capacity(cache_entry_records.len());
        for entry in cache_entry_records {
            let (Ok(key), Ok(value)) = (u32::try_from(entry.key), u32::try_from(entry.value))
            else {
                continue;
            };
            cache_entries.push((key, value));
        }
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
            && radiance_cache_gpu_stage_dispatch_counts
                .iter()
                .all(|count| *count == 0)
            && global_sdf_stats.is_none()
            && scene_prepare.is_none()
        {
            return None;
        }

        Some(
            Self::new(
                cache_entries,
                completed_probe_ids,
                outputs.completed_trace_region_ids,
                probe_irradiance_rgb,
                probe_trace_lighting_rgb,
                scene_prepare,
            )
            .with_radiance_cache_gpu_stage_dispatch_counts(radiance_cache_gpu_stage_dispatch_counts)
            .with_global_sdf_stats(global_sdf_stats),
        )
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
        RenderHybridGiCacheEntryRecord, RenderHybridGiGlobalSdfStats,
        RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
        RenderHybridGiScenePrepareSample, RenderHybridGiVoxelCellSampleRecord,
        RenderHybridGiVoxelOccupancyMaskRecord,
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
                radiance_cache_gpu_stage_dispatch_counts: Default::default(),
                global_sdf_stats: None,
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
    fn gpu_completion_keeps_gpu_authored_radiance_cache_dispatch_counts() {
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                radiance_cache_gpu_stage_dispatch_counts: [1, 1, 1, 1, 1, 2],
                ..RenderHybridGiReadbackOutputs::default()
            })
            .expect("GPU-authored dispatch counts are runtime feedback");

        assert_eq!(
            completion.radiance_cache_gpu_stage_dispatch_counts(),
            [1, 1, 1, 1, 1, 2]
        );
    }

    #[test]
    fn gpu_completion_keeps_global_sdf_runtime_stats() {
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                    cpu_prepare_time_us: 1500,
                    cpu_mesh_object_collection_time_us: 200,
                    cpu_mesh_scene_sync_time_us: 300,
                    cpu_residency_time_us: 400,
                    cpu_influence_update_time_us: 100,
                    cpu_candidate_build_time_us: 500,
                    mesh_projection_cache_hit: true,
                    object_count: 17,
                    resident_page_count: 9,
                    dirty_page_count: 3,
                    uploaded_page_count: 2,
                    candidate_overflow_page_count: 1,
                    candidate_contributor_count: 12,
                    clipmap_fallback_count: 1,
                    candidate_bucket_capacity_bytes: 256,
                    persistent_resource_byte_count: 4096,
                    transient_upload_byte_count: 256,
                    ..RenderHybridGiGlobalSdfStats::default()
                }),
                ..RenderHybridGiReadbackOutputs::default()
            })
            .expect("Global SDF stats must be runtime feedback");

        let stats = completion
            .global_sdf_stats()
            .expect("Global SDF stats must survive completion projection");
        assert_eq!(stats.object_count, 17);
        assert_eq!(stats.cpu_influence_update_time_us, 100);
        assert!(stats.mesh_projection_cache_hit);
        assert_eq!(stats.uploaded_page_count, 2);
        assert_eq!(stats.candidate_overflow_page_count, 1);
        assert_eq!(stats.candidate_contributor_count, 12);
        assert_eq!(stats.clipmap_fallback_count, 1);
        assert_eq!(stats.candidate_bucket_capacity_bytes, 256);
        assert_eq!(stats.persistent_resource_byte_count, 4096);
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

    #[test]
    fn gpu_completion_preallocates_filtered_cache_projection() {
        let source = include_str!("gpu_completion.rs");
        let capacity = concat!("Vec::with_capacity(", "cache_entry_records.len())");

        assert!(source.contains(capacity));
    }

    #[test]
    fn gpu_completion_skips_cache_entries_outside_runtime_id_range() {
        let overflow = u64::from(u32::MAX) + 1;
        let completion =
            HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
                cache_entries: vec![
                    RenderHybridGiCacheEntryRecord { key: 17, value: 3 },
                    RenderHybridGiCacheEntryRecord {
                        key: overflow,
                        value: 4,
                    },
                    RenderHybridGiCacheEntryRecord {
                        key: 19,
                        value: overflow,
                    },
                ],
                ..RenderHybridGiReadbackOutputs::default()
            })
            .expect("valid cache entry should keep completion");

        assert_eq!(completion.cache_entries(), &[(17, 3)]);
    }
}
