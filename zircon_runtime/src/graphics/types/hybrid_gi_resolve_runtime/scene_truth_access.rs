use super::packing::unpack_rgb_and_weight;
use super::resolve_runtime::HybridGiResolveRuntime;

impl HybridGiResolveRuntime {
    pub(crate) fn scene_truth_irradiance_probe_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.probe_scene_driven_hierarchy_irradiance_ids
            .iter()
            .copied()
    }

    pub(crate) fn scene_truth_rt_lighting_probe_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.probe_scene_driven_hierarchy_rt_lighting_ids
            .iter()
            .copied()
    }

    pub(crate) fn hierarchy_resolve_weight(&self, probe_id: u32) -> Option<f32> {
        self.probe_hierarchy_resolve_weight_q8
            .get(&probe_id)
            .map(|weight_q8| *weight_q8 as f32 / 256.0)
    }

    pub(crate) fn has_hierarchy_resolve_weight(&self, probe_id: u32) -> bool {
        self.probe_hierarchy_resolve_weight_q8
            .contains_key(&probe_id)
    }

    pub(crate) fn hierarchy_irradiance(&self, probe_id: u32) -> Option<[f32; 4]> {
        self.probe_hierarchy_irradiance_rgb_and_weight
            .get(&probe_id)
            .copied()
            .map(unpack_rgb_and_weight)
    }

    pub(crate) fn hierarchy_irradiance_includes_scene_truth(&self, probe_id: u32) -> bool {
        self.probe_scene_driven_hierarchy_irradiance_ids
            .contains(&probe_id)
    }

    pub(crate) fn hierarchy_irradiance_scene_truth_quality(&self, probe_id: u32) -> f32 {
        self.probe_scene_driven_hierarchy_irradiance_quality_q8
            .get(&probe_id)
            .copied()
            .map(|quality_q8| quality_q8 as f32 / 255.0)
            .unwrap_or(1.0)
    }

    pub(crate) fn hierarchy_irradiance_scene_truth_freshness(&self, probe_id: u32) -> f32 {
        self.probe_scene_driven_hierarchy_irradiance_freshness_q8
            .get(&probe_id)
            .copied()
            .map(|freshness_q8| freshness_q8 as f32 / 255.0)
            .unwrap_or(1.0)
    }

    pub(crate) fn hierarchy_irradiance_scene_truth_revision(&self, probe_id: u32) -> u32 {
        self.probe_scene_driven_hierarchy_irradiance_revision
            .get(&probe_id)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn hierarchy_rt_lighting(&self, probe_id: u32) -> Option<[f32; 4]> {
        self.probe_hierarchy_rt_lighting_rgb_and_weight
            .get(&probe_id)
            .copied()
            .map(unpack_rgb_and_weight)
    }

    pub(crate) fn hierarchy_rt_lighting_includes_scene_truth(&self, probe_id: u32) -> bool {
        self.probe_scene_driven_hierarchy_rt_lighting_ids
            .contains(&probe_id)
    }

    pub(crate) fn hierarchy_rt_lighting_scene_truth_quality(&self, probe_id: u32) -> f32 {
        self.probe_scene_driven_hierarchy_rt_lighting_quality_q8
            .get(&probe_id)
            .copied()
            .map(|quality_q8| quality_q8 as f32 / 255.0)
            .unwrap_or(1.0)
    }

    pub(crate) fn hierarchy_rt_lighting_scene_truth_freshness(&self, probe_id: u32) -> f32 {
        self.probe_scene_driven_hierarchy_rt_lighting_freshness_q8
            .get(&probe_id)
            .copied()
            .map(|freshness_q8| freshness_q8 as f32 / 255.0)
            .unwrap_or(1.0)
    }

    pub(crate) fn hierarchy_rt_lighting_scene_truth_revision(&self, probe_id: u32) -> u32 {
        self.probe_scene_driven_hierarchy_rt_lighting_revision
            .get(&probe_id)
            .copied()
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub(crate) fn remove_hierarchy_irradiance_for_test(&mut self, probe_id: u32) {
        self.probe_hierarchy_irradiance_rgb_and_weight
            .remove(&probe_id);
    }

    #[cfg(test)]
    pub(crate) fn remove_hierarchy_rt_lighting_for_test(&mut self, probe_id: u32) {
        self.probe_hierarchy_rt_lighting_rgb_and_weight
            .remove(&probe_id);
    }

    #[cfg(test)]
    pub(crate) fn remove_hierarchy_resolve_weight_for_test(&mut self, probe_id: u32) {
        self.probe_hierarchy_resolve_weight_q8.remove(&probe_id);
    }
}
