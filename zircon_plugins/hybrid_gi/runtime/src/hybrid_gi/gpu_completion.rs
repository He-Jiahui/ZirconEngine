use crate::hybrid_gi::HybridGiRuntimeScenePrepareResources;

pub(crate) struct HybridGiGpuCompletion {
    cache_entries: Vec<(u32, u32)>,
    completed_probe_ids: Vec<u32>,
    completed_trace_region_ids: Vec<u32>,
    probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    scene_prepare_resources: Option<HybridGiRuntimeScenePrepareResources>,
}

impl HybridGiGpuCompletion {
    pub(crate) fn new(
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

    pub(crate) fn cache_entries(&self) -> &[(u32, u32)] {
        &self.cache_entries
    }

    pub(crate) fn completed_probe_ids(&self) -> &[u32] {
        &self.completed_probe_ids
    }

    pub(crate) fn completed_trace_region_ids(&self) -> &[u32] {
        &self.completed_trace_region_ids
    }

    pub(crate) fn probe_irradiance_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_irradiance_rgb
    }

    pub(crate) fn probe_trace_lighting_rgb(&self) -> &[(u32, [u8; 3])] {
        &self.probe_trace_lighting_rgb
    }

    pub(crate) fn scene_prepare_resources(&self) -> Option<&HybridGiRuntimeScenePrepareResources> {
        self.scene_prepare_resources.as_ref()
    }
}
