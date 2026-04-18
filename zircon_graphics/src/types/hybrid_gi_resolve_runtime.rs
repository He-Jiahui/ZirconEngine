use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiResolveRuntime {
    pub(crate) probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    pub(crate) probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    pub(crate) probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    pub(crate) probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
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

    pub(crate) fn hierarchy_rt_lighting(&self, probe_id: u32) -> Option<[f32; 4]> {
        self.probe_hierarchy_rt_lighting_rgb_and_weight
            .get(&probe_id)
            .copied()
            .map(unpack_rgb_and_weight)
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
