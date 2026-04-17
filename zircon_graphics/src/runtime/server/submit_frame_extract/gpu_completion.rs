pub(super) struct HybridGiGpuCompletion {
    pub(super) cache_entries: Vec<(u32, u32)>,
    pub(super) completed_probe_ids: Vec<u32>,
    pub(super) completed_trace_region_ids: Vec<u32>,
    pub(super) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
}

pub(super) struct VirtualGeometryGpuCompletion {
    pub(super) page_table_entries: Vec<(u32, u32)>,
    pub(super) completed_page_assignments: Vec<(u32, u32)>,
}
