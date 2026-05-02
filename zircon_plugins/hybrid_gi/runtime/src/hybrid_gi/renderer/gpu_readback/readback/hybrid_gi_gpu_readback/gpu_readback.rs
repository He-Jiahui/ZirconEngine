use super::HybridGiScenePrepareResourcesSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(super) cache_entries: Vec<(u32, u32)>,
    pub(super) completed_probe_ids: Vec<u32>,
    pub(super) completed_trace_region_ids: Vec<u32>,
    pub(super) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    pub(super) probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    pub(super) scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
}
