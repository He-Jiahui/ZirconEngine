use std::collections::BTreeMap;

use super::probe_scene_data::HybridGiRuntimeProbeSceneData;
use super::runtime_state::HybridGiRuntimeState;
use super::trace_region_scene_data::HybridGiRuntimeTraceRegionSceneData;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn trace_region_scene_data(
        &self,
    ) -> &BTreeMap<u32, HybridGiRuntimeTraceRegionSceneData> {
        &self.trace_region_scene_data
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn trace_region_scene_data_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, HybridGiRuntimeTraceRegionSceneData> {
        &mut self.trace_region_scene_data
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_scene_data(
        &self,
    ) -> &BTreeMap<u32, HybridGiRuntimeProbeSceneData> {
        &self.probe_scene_data
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_scene_data_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, HybridGiRuntimeProbeSceneData> {
        &mut self.probe_scene_data
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_irradiance_rgb(
        &self,
    ) -> &BTreeMap<u32, [u8; 3]> {
        &self.probe_irradiance_rgb
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_irradiance_rgb_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, [u8; 3]> {
        &mut self.probe_irradiance_rgb
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_rt_lighting_rgb(
        &self,
    ) -> &BTreeMap<u32, [u8; 3]> {
        &self.probe_rt_lighting_rgb
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_rt_lighting_rgb_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, [u8; 3]> {
        &mut self.probe_rt_lighting_rgb
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_ray_budgets(&self) -> &BTreeMap<u32, u32> {
        &self.probe_ray_budgets
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_ray_budgets_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, u32> {
        &mut self.probe_ray_budgets
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_parent_probes(
        &self,
    ) -> &BTreeMap<u32, u32> {
        &self.probe_parent_probes
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn probe_parent_probes_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, u32> {
        &mut self.probe_parent_probes
    }
}
