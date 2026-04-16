#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityHybridGiFeedback {
    pub active_probe_ids: Vec<u32>,
    pub requested_probe_ids: Vec<u32>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
}
