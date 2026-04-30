use super::probe_scene_data::HybridGiResolveProbeSceneData;
use super::resolve_runtime::HybridGiResolveRuntime;
use super::trace_region_scene_data::HybridGiResolveTraceRegionSceneData;

impl HybridGiResolveRuntime {
    pub(crate) fn probe_scene_data(&self, probe_id: u32) -> Option<HybridGiResolveProbeSceneData> {
        self.probe_scene_data.get(&probe_id).copied()
    }

    pub(crate) fn trace_region_scene_data(
        &self,
        region_id: u32,
    ) -> Option<HybridGiResolveTraceRegionSceneData> {
        self.trace_region_scene_data.get(&region_id).copied()
    }

    #[cfg(test)]
    pub(crate) fn has_probe_scene_data_entries(&self) -> bool {
        !self.probe_scene_data.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn has_trace_region_scene_data_entries(&self) -> bool {
        !self.trace_region_scene_data.is_empty()
    }

    pub(crate) fn probe_rt_lighting_rgb(&self, probe_id: u32) -> Option<[u8; 3]> {
        self.probe_rt_lighting_rgb.get(&probe_id).copied()
    }

    pub(crate) fn has_probe_rt_lighting(&self, probe_id: u32) -> bool {
        self.probe_rt_lighting_rgb(probe_id).is_some()
    }
}
