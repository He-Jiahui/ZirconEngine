use std::collections::{BTreeMap, BTreeSet};

use super::probe_scene_data::HybridGiResolveProbeSceneData;
use super::resolve_runtime::HybridGiResolveRuntime;
use super::trace_region_scene_data::HybridGiResolveTraceRegionSceneData;

#[derive(Default)]
pub(crate) struct HybridGiResolveRuntimeTestBuilder {
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
}

impl HybridGiResolveRuntime {
    pub(crate) fn fixture() -> HybridGiResolveRuntimeTestBuilder {
        HybridGiResolveRuntimeTestBuilder::new()
    }
}

#[allow(dead_code)]
impl HybridGiResolveRuntimeTestBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_probe_scene_data(
        mut self,
        probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
    ) -> Self {
        self.probe_scene_data = probe_scene_data;
        self
    }

    pub(crate) fn with_trace_region_scene_data(
        mut self,
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> Self {
        self.trace_region_scene_data = trace_region_scene_data;
        self
    }

    pub(crate) fn with_probe_parent_probes(
        mut self,
        probe_parent_probes: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_parent_probes = probe_parent_probes;
        self
    }

    pub(crate) fn with_probe_rt_lighting_rgb(
        mut self,
        probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    ) -> Self {
        self.probe_rt_lighting_rgb = probe_rt_lighting_rgb;
        self
    }

    pub(crate) fn with_probe_hierarchy_resolve_weight_q8(
        mut self,
        probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    ) -> Self {
        self.probe_hierarchy_resolve_weight_q8 = probe_hierarchy_resolve_weight_q8;
        self
    }

    pub(crate) fn with_probe_hierarchy_irradiance_rgb_and_weight(
        mut self,
        probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    ) -> Self {
        self.probe_hierarchy_irradiance_rgb_and_weight = probe_hierarchy_irradiance_rgb_and_weight;
        self
    }

    pub(crate) fn with_probe_hierarchy_rt_lighting_rgb_and_weight(
        mut self,
        probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    ) -> Self {
        self.probe_hierarchy_rt_lighting_rgb_and_weight =
            probe_hierarchy_rt_lighting_rgb_and_weight;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_ids(
        mut self,
        probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_ids =
            probe_scene_driven_hierarchy_irradiance_ids;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_ids(
        mut self,
        probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_ids =
            probe_scene_driven_hierarchy_rt_lighting_ids;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_quality_q8(
        mut self,
        quality_q8: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_quality_q8 = quality_q8;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_freshness_q8(
        mut self,
        freshness_q8: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_freshness_q8 = freshness_q8;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_revision(
        mut self,
        revision: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_revision = revision;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_quality_q8(
        mut self,
        quality_q8: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_quality_q8 = quality_q8;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_freshness_q8(
        mut self,
        freshness_q8: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_freshness_q8 = freshness_q8;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_revision(
        mut self,
        revision: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_revision = revision;
        self
    }

    pub(crate) fn build(self) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::new(
            self.probe_scene_data,
            self.trace_region_scene_data,
            self.probe_parent_probes,
            self.probe_rt_lighting_rgb,
            self.probe_hierarchy_resolve_weight_q8,
            self.probe_hierarchy_irradiance_rgb_and_weight,
            self.probe_hierarchy_rt_lighting_rgb_and_weight,
            self.probe_scene_driven_hierarchy_irradiance_ids,
            self.probe_scene_driven_hierarchy_rt_lighting_ids,
            self.probe_scene_driven_hierarchy_irradiance_quality_q8,
            self.probe_scene_driven_hierarchy_irradiance_freshness_q8,
            self.probe_scene_driven_hierarchy_irradiance_revision,
            self.probe_scene_driven_hierarchy_rt_lighting_quality_q8,
            self.probe_scene_driven_hierarchy_rt_lighting_freshness_q8,
            self.probe_scene_driven_hierarchy_rt_lighting_revision,
        )
    }
}
