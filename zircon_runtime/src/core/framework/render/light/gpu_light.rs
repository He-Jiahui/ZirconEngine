use bytemuck::{Pod, Zeroable};

pub const SHADOW_SLOT_NONE: u32 = u32::MAX;
pub const GPU_LIGHT_DATA_STRIDE: usize = 128;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// 灯光上传 DTO 固定为 `repr(C, align(16))` 的 128 字节步长，与 WGSL/std430 缓冲布局对齐；类型位及阴影、cookie 槽位由打包器解释。
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
#[path = "tests/gpu_light.rs"]
mod tests;
