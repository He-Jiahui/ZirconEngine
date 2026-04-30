use super::HybridGiScenePrepareResourcesSnapshot;

pub(in crate::graphics) struct HybridGiGpuReadbackCompletionParts {
    cache_entries: Vec<(u32, u32)>,
    completed_probe_ids: Vec<u32>,
    completed_trace_region_ids: Vec<u32>,
    probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    scene_prepare_surface_cache_samples: Option<(Vec<(u32, [u8; 4])>, Vec<(u32, [u8; 4])>)>,
}

impl HybridGiGpuReadbackCompletionParts {
    pub(super) fn new(
        cache_entries: Vec<(u32, u32)>,
        completed_probe_ids: Vec<u32>,
        completed_trace_region_ids: Vec<u32>,
        probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
        probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
        scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
    ) -> Self {
        Self {
            cache_entries,
            completed_probe_ids,
            completed_trace_region_ids,
            probe_irradiance_rgb,
            probe_trace_lighting_rgb,
            scene_prepare_surface_cache_samples: scene_prepare_resources
                .map(HybridGiScenePrepareResourcesSnapshot::into_surface_cache_samples),
        }
    }

    pub(in crate::graphics) fn into_parts(
        self,
    ) -> (
        Vec<(u32, u32)>,
        Vec<u32>,
        Vec<u32>,
        Vec<(u32, [u8; 3])>,
        Vec<(u32, [u8; 3])>,
        Option<(Vec<(u32, [u8; 4])>, Vec<(u32, [u8; 4])>)>,
    ) {
        (
            self.cache_entries,
            self.completed_probe_ids,
            self.completed_trace_region_ids,
            self.probe_irradiance_rgb,
            self.probe_trace_lighting_rgb,
            self.scene_prepare_surface_cache_samples,
        )
    }
}
