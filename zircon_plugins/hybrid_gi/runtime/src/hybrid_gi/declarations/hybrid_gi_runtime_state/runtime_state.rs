use std::collections::{BTreeMap, BTreeSet};

use super::super::hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;
use super::probe_scene_data::HybridGiRuntimeProbeSceneData;
use super::trace_region_scene_data::HybridGiRuntimeTraceRegionSceneData;
use crate::hybrid_gi::scene_representation::HybridGiSceneRepresentation;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HybridGiRuntimeState {
    pub(super) scene_representation: HybridGiSceneRepresentation,
    pub(super) probe_budget: usize,
    pub(super) probe_parent_probes: BTreeMap<u32, u32>,
    pub(super) probe_child_probes: BTreeMap<u32, Vec<u32>>,
    pub(super) probe_ray_budgets: BTreeMap<u32, u32>,
    pub(super) probe_scene_data: BTreeMap<u32, HybridGiRuntimeProbeSceneData>,
    pub(super) probe_irradiance_rgb: BTreeMap<u32, [u8; 3]>,
    pub(super) probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    pub(super) trace_region_scene_data: BTreeMap<u32, HybridGiRuntimeTraceRegionSceneData>,
    pub(super) resident_slots: BTreeMap<u32, u32>,
    pub(super) pending_updates: Vec<HybridGiProbeUpdateRequest>,
    pub(super) pending_probes: BTreeSet<u32>,
    pub(super) scheduled_trace_regions: Vec<u32>,
    pub(super) current_requested_probe_ids: BTreeSet<u32>,
    pub(super) recent_lineage_trace_support_q8: BTreeMap<u32, u16>,
    pub(super) recent_requested_lineage_support_q8: BTreeMap<u32, u16>,
    pub(super) evictable_probes: Vec<u32>,
    pub(super) free_slots: BTreeSet<u32>,
    pub(super) next_slot: u32,
}
