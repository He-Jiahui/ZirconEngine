#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(crate) cache_entries: Vec<(u32, u32)>,
    pub(crate) completed_probe_ids: Vec<u32>,
    pub(crate) completed_trace_region_ids: Vec<u32>,
    pub(crate) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
}
