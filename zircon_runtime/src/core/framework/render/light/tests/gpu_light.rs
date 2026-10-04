use super::*;
use std::mem::{offset_of, size_of};

#[test]
fn gpu_light_data_layout_matches_plan_05_std430_contract() {
    assert_eq!(size_of::<GpuLightData>(), GPU_LIGHT_DATA_STRIDE);
    assert_eq!(offset_of!(GpuLightData, position_range), 0);
    assert_eq!(offset_of!(GpuLightData, color_intensity), 16);
    assert_eq!(offset_of!(GpuLightData, direction_type), 32);
    assert_eq!(offset_of!(GpuLightData, spot_angles_size), 48);
    assert_eq!(offset_of!(GpuLightData, shadow_slot_layer), 64);
    assert_eq!(offset_of!(GpuLightData, shadow_params), 80);
    assert_eq!(offset_of!(GpuLightData, cookie_uv_rect), 96);
    assert_eq!(offset_of!(GpuLightData, cookie_misc), 112);
}

#[test]
fn gpu_light_type_is_encoded_as_bits_for_wgsl_bitcast() {
    assert_eq!(GpuLightType::Directional.as_f32_bits().to_bits(), 0);
    assert_eq!(GpuLightType::Point.as_f32_bits().to_bits(), 1);
    assert_eq!(GpuLightType::Spot.as_f32_bits().to_bits(), 2);
    assert_eq!(GpuLightType::Rect.as_f32_bits().to_bits(), 3);
}
