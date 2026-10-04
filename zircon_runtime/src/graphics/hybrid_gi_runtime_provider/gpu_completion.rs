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
#[path = "tests/gpu_completion.rs"]
mod tests;
