#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeSnapshot {
    pub(crate) cache_entry_count: usize,
    pub(crate) resident_probe_count: usize,
    pub(crate) pending_update_count: usize,
    pub(crate) scheduled_trace_region_count: usize,
}
