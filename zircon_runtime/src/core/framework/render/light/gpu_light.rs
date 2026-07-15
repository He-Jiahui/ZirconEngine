use bytemuck::{Pod, Zeroable};

pub const SHADOW_SLOT_NONE: u32 = u32::MAX;
pub const GPU_LIGHT_DATA_STRIDE: usize = 128;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuLightType {
    Directional = 0,
    Point = 1,
    Spot = 2,
    Rect = 3,
}

impl GpuLightType {
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn as_f32_bits(self) -> f32 {
        f32::from_bits(self.as_u32())
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuLightData {
    pub position_range: [f32; 4],
    pub color_intensity: [f32; 4],
    pub direction_type: [f32; 4],
    pub spot_angles_size: [f32; 4],
    pub shadow_slot_layer: [u32; 4],
    pub shadow_params: [f32; 4],
    pub cookie_uv_rect: [f32; 4],
    pub cookie_misc: [u32; 4],
}

impl GpuLightData {
    pub const STRIDE: usize = GPU_LIGHT_DATA_STRIDE;
}

#[cfg(test)]
mod tests {
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
}
