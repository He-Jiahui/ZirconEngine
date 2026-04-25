use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiResolveRuntime {
    pub(crate) probe_parent_probes: BTreeMap<u32, u32>,
    pub(crate) probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    pub(crate) probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    pub(crate) probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    pub(crate) probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    pub(crate) probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
    pub(crate) probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
    pub(crate) probe_scene_driven_hierarchy_irradiance_quality_q8: BTreeMap<u32, u8>,
    pub(crate) probe_scene_driven_hierarchy_irradiance_freshness_q8: BTreeMap<u32, u8>,
    pub(crate) probe_scene_driven_hierarchy_irradiance_revision: BTreeMap<u32, u32>,
    pub(crate) probe_scene_driven_hierarchy_rt_lighting_quality_q8: BTreeMap<u32, u8>,
    pub(crate) probe_scene_driven_hierarchy_rt_lighting_freshness_q8: BTreeMap<u32, u8>,
    pub(crate) probe_scene_driven_hierarchy_rt_lighting_revision: BTreeMap<u32, u32>,
}

impl HybridGiResolveRuntime {
    pub(crate) fn hierarchy_resolve_weight(&self, probe_id: u32) -> Option<f32> {
        self.probe_hierarchy_resolve_weight_q8
            .get(&probe_id)
            .map(|weight_q8| *weight_q8 as f32 / 256.0)
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

    pub(crate) fn pack_rgb_and_weight(rgb: [f32; 3], weight: f32) -> [u8; 4] {
        [
            quantize_unit_float(rgb[0]),
            quantize_unit_float(rgb[1]),
            quantize_unit_float(rgb[2]),
            quantize_unit_float(weight),
        ]
    }

    pub(crate) fn pack_resolve_weight_q8(weight: f32) -> u16 {
        (weight.clamp(0.0, u16::MAX as f32 / 256.0) * 256.0).round() as u16
    }

    pub(crate) fn pack_scene_truth_quality_q8(quality: f32) -> u8 {
        quantize_unit_float(quality)
    }

    pub(crate) fn pack_scene_truth_freshness_q8(freshness: f32) -> u8 {
        quantize_unit_float(freshness)
    }
}

fn unpack_rgb_and_weight(packed: [u8; 4]) -> [f32; 4] {
    [
        packed[0] as f32 / 255.0,
        packed[1] as f32 / 255.0,
        packed[2] as f32 / 255.0,
        packed[3] as f32 / 255.0,
    ]
}

fn quantize_unit_float(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
