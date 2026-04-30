use super::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};

#[cfg_attr(not(test), allow(dead_code))]
impl HybridGiGpuReadback {
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

    pub(crate) fn scene_prepare_resources(&self) -> Option<HybridGiScenePrepareResourcesSnapshot> {
        self.scene_prepare_resources.clone()
    }
}
