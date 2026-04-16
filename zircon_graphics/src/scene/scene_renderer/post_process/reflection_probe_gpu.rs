use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuReflectionProbe {
    pub(super) screen_uv_and_radius: [f32; 4],
    pub(super) color_and_intensity: [f32; 4],
}
