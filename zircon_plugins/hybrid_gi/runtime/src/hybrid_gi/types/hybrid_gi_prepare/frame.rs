use super::{HybridGiPrepareProbe, HybridGiPrepareUpdateRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiPrepareFrame {
    pub resident_probes: Vec<HybridGiPrepareProbe>,
    pub pending_updates: Vec<HybridGiPrepareUpdateRequest>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
}
