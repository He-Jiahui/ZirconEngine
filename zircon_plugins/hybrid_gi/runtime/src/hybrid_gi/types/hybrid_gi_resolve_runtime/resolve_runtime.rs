use std::collections::{BTreeMap, BTreeSet};

use super::probe_scene_data::HybridGiResolveProbeSceneData;
use super::trace_region_scene_data::HybridGiResolveTraceRegionSceneData;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiResolveRuntime {
    pub(super) probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
    pub(super) trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    pub(super) probe_parent_probes: BTreeMap<u32, u32>,
    pub(super) probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    pub(super) probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    pub(super) probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    pub(super) probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    pub(super) probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
    pub(super) probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
    pub(super) probe_scene_driven_hierarchy_irradiance_quality_q8: BTreeMap<u32, u8>,
    pub(super) probe_scene_driven_hierarchy_irradiance_freshness_q8: BTreeMap<u32, u8>,
    pub(super) probe_scene_driven_hierarchy_irradiance_revision: BTreeMap<u32, u32>,
    pub(super) probe_scene_driven_hierarchy_rt_lighting_quality_q8: BTreeMap<u32, u8>,
    pub(super) probe_scene_driven_hierarchy_rt_lighting_freshness_q8: BTreeMap<u32, u8>,
    pub(super) probe_scene_driven_hierarchy_rt_lighting_revision: BTreeMap<u32, u32>,
}

impl HybridGiResolveRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
        probe_parent_probes: BTreeMap<u32, u32>,
        probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
        probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
        probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
        probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
        probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
        probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
        probe_scene_driven_hierarchy_irradiance_quality_q8: BTreeMap<u32, u8>,
        probe_scene_driven_hierarchy_irradiance_freshness_q8: BTreeMap<u32, u8>,
        probe_scene_driven_hierarchy_irradiance_revision: BTreeMap<u32, u32>,
        probe_scene_driven_hierarchy_rt_lighting_quality_q8: BTreeMap<u32, u8>,
        probe_scene_driven_hierarchy_rt_lighting_freshness_q8: BTreeMap<u32, u8>,
        probe_scene_driven_hierarchy_rt_lighting_revision: BTreeMap<u32, u32>,
    ) -> Self {
        Self {
            probe_scene_data,
            trace_region_scene_data,
            probe_parent_probes,
            probe_rt_lighting_rgb,
            probe_hierarchy_resolve_weight_q8,
            probe_hierarchy_irradiance_rgb_and_weight,
            probe_hierarchy_rt_lighting_rgb_and_weight,
            probe_scene_driven_hierarchy_irradiance_ids,
            probe_scene_driven_hierarchy_rt_lighting_ids,
            probe_scene_driven_hierarchy_irradiance_quality_q8,
            probe_scene_driven_hierarchy_irradiance_freshness_q8,
            probe_scene_driven_hierarchy_irradiance_revision,
            probe_scene_driven_hierarchy_rt_lighting_quality_q8,
            probe_scene_driven_hierarchy_rt_lighting_freshness_q8,
            probe_scene_driven_hierarchy_rt_lighting_revision,
        }
    }
}
