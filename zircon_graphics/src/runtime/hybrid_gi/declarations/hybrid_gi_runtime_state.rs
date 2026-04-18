use std::collections::{BTreeMap, BTreeSet};

use super::hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeState {
    pub(in crate::runtime::hybrid_gi) probe_budget: usize,
    pub(in crate::runtime::hybrid_gi) probe_ray_budgets: BTreeMap<u32, u32>,
    pub(in crate::runtime::hybrid_gi) probe_irradiance_rgb: BTreeMap<u32, [u8; 3]>,
    pub(in crate::runtime::hybrid_gi) resident_slots: BTreeMap<u32, u32>,
    pub(in crate::runtime::hybrid_gi) pending_updates: Vec<HybridGiProbeUpdateRequest>,
    pub(in crate::runtime::hybrid_gi) pending_probes: BTreeSet<u32>,
    pub(in crate::runtime::hybrid_gi) scheduled_trace_regions: Vec<u32>,
    pub(in crate::runtime::hybrid_gi) evictable_probes: Vec<u32>,
    pub(in crate::runtime::hybrid_gi) free_slots: BTreeSet<u32>,
    pub(in crate::runtime::hybrid_gi) next_slot: u32,
}
