use std::collections::{BTreeMap, BTreeSet};

use super::hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime::hybrid_gi) struct HybridGiRuntimeProbeSceneData {
    pub(in crate::runtime::hybrid_gi) position_x_q: u32,
    pub(in crate::runtime::hybrid_gi) position_y_q: u32,
    pub(in crate::runtime::hybrid_gi) position_z_q: u32,
    pub(in crate::runtime::hybrid_gi) radius_q: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime::hybrid_gi) struct HybridGiRuntimeTraceRegionSceneData {
    pub(in crate::runtime::hybrid_gi) center_x_q: u32,
    pub(in crate::runtime::hybrid_gi) center_y_q: u32,
    pub(in crate::runtime::hybrid_gi) center_z_q: u32,
    pub(in crate::runtime::hybrid_gi) radius_q: u32,
    pub(in crate::runtime::hybrid_gi) coverage_q: u32,
    pub(in crate::runtime::hybrid_gi) rt_lighting_rgb: [u8; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeState {
    pub(in crate::runtime::hybrid_gi) probe_budget: usize,
    pub(in crate::runtime::hybrid_gi) probe_parent_probes: BTreeMap<u32, u32>,
    pub(in crate::runtime::hybrid_gi) probe_ray_budgets: BTreeMap<u32, u32>,
    pub(in crate::runtime::hybrid_gi) probe_scene_data:
        BTreeMap<u32, HybridGiRuntimeProbeSceneData>,
    pub(in crate::runtime::hybrid_gi) probe_irradiance_rgb: BTreeMap<u32, [u8; 3]>,
    pub(in crate::runtime::hybrid_gi) probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    pub(in crate::runtime::hybrid_gi) trace_region_scene_data:
        BTreeMap<u32, HybridGiRuntimeTraceRegionSceneData>,
    pub(in crate::runtime::hybrid_gi) resident_slots: BTreeMap<u32, u32>,
    pub(in crate::runtime::hybrid_gi) pending_updates: Vec<HybridGiProbeUpdateRequest>,
    pub(in crate::runtime::hybrid_gi) pending_probes: BTreeSet<u32>,
    pub(in crate::runtime::hybrid_gi) scheduled_trace_regions: Vec<u32>,
    pub(in crate::runtime::hybrid_gi) current_requested_probe_ids: BTreeSet<u32>,
    pub(in crate::runtime::hybrid_gi) recent_lineage_trace_support_q8:
        BTreeMap<u32, u16>,
    pub(in crate::runtime::hybrid_gi) recent_requested_lineage_support_q8:
        BTreeMap<u32, u16>,
    pub(in crate::runtime::hybrid_gi) evictable_probes: Vec<u32>,
    pub(in crate::runtime::hybrid_gi) free_slots: BTreeSet<u32>,
    pub(in crate::runtime::hybrid_gi) next_slot: u32,
}
