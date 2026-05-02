use super::super::HybridGiGpuReadbackCompletionParts;
use super::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};

impl HybridGiGpuReadback {
    pub(in crate::hybrid_gi::renderer) fn new(
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
            scene_prepare_resources,
        }
    }

    pub(in crate::hybrid_gi::renderer) fn into_completion_parts(
        self,
    ) -> HybridGiGpuReadbackCompletionParts {
        HybridGiGpuReadbackCompletionParts::new(
            self.cache_entries,
            self.completed_probe_ids,
            self.completed_trace_region_ids,
            self.probe_irradiance_rgb,
            self.probe_trace_lighting_rgb,
            self.scene_prepare_resources,
        )
    }
}
