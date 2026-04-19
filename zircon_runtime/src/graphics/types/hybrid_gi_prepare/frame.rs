use super::{HybridGiPrepareProbe, HybridGiPrepareUpdateRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareFrame {
    pub(crate) resident_probes: Vec<HybridGiPrepareProbe>,
    pub(crate) pending_updates: Vec<HybridGiPrepareUpdateRequest>,
    pub(crate) scheduled_trace_region_ids: Vec<u32>,
    pub(crate) evictable_probe_ids: Vec<u32>,
}
