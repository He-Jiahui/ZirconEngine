use super::resolve_runtime::HybridGiResolveRuntime;

impl HybridGiResolveRuntime {
    pub fn pack_rgb_and_weight(rgb: [f32; 3], weight: f32) -> [u8; 4] {
        [
            quantize_unit_float(rgb[0]),
            quantize_unit_float(rgb[1]),
            quantize_unit_float(rgb[2]),
            quantize_unit_float(weight),
        ]
    }

    pub fn pack_resolve_weight_q8(weight: f32) -> u16 {
        (weight.clamp(0.0, u16::MAX as f32 / 256.0) * 256.0).round() as u16
    }

    pub fn pack_scene_truth_quality_q8(quality: f32) -> u8 {
        quantize_unit_float(quality)
    }

    pub fn pack_scene_truth_freshness_q8(freshness: f32) -> u8 {
        quantize_unit_float(freshness)
    }
}

pub(super) fn unpack_rgb_and_weight(packed: [u8; 4]) -> [f32; 4] {
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
