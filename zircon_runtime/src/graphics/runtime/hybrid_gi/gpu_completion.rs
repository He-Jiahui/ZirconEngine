use crate::graphics::runtime::hybrid_gi::HybridGiRuntimeScenePrepareResources;

pub(in crate::graphics::runtime) struct HybridGiGpuCompletion {
    cache_entries: Vec<(u32, u32)>,
    completed_probe_ids: Vec<u32>,
    completed_trace_region_ids: Vec<u32>,
    probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    scene_prepare_resources: Option<HybridGiRuntimeScenePrepareResources>,
}

impl HybridGiGpuCompletion {
    pub(in crate::graphics::runtime) fn new(
        cache_entries: Vec<(u32, u32)>,
        completed_probe_ids: Vec<u32>,
        completed_trace_region_ids: Vec<u32>,
        probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
        probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
        scene_prepare_resources: Option<HybridGiRuntimeScenePrepareResources>,
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

    pub(in crate::graphics::runtime) fn cache_entries(&self) -> &[(u32, u32)] {
        &self.cache_entries
    }

    pub(in crate::graphics::runtime) fn completed_probe_ids(&self) -> &[u32] {
        &self.completed_probe_ids
    }

    pub(in crate::graphics::runtime) fn completed_trace_region_ids(&self) -> &[u32] {
        &self.completed_trace_region_ids
    }

    pub(in crate::graphics::runtime) fn probe_irradiance_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_irradiance_rgb
    }

    pub(in crate::graphics::runtime) fn probe_trace_lighting_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_trace_lighting_rgb
    }

    pub(in crate::graphics::runtime) fn scene_prepare_resources(
        &self,
    ) -> Option<&HybridGiRuntimeScenePrepareResources> {
        self.scene_prepare_resources.as_ref()
    }
}
