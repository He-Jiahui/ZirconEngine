use std::collections::{BTreeMap, BTreeSet};

use super::hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeState {
    pub(super) probe_budget: usize,
    pub(super) probe_ray_budgets: BTreeMap<u32, u32>,
    pub(super) probe_irradiance_rgb: BTreeMap<u32, [u8; 3]>,
    pub(super) resident_slots: BTreeMap<u32, u32>,
    pub(super) pending_updates: Vec<HybridGiProbeUpdateRequest>,
    pub(super) pending_probes: BTreeSet<u32>,
    pub(super) scheduled_trace_regions: Vec<u32>,
    pub(super) evictable_probes: Vec<u32>,
    pub(super) free_slots: BTreeSet<u32>,
    pub(super) next_slot: u32,
}
