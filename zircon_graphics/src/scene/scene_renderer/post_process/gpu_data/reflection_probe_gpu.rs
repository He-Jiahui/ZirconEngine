use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::post_process) struct GpuReflectionProbe {
    pub(in crate::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::scene::scene_renderer::post_process) color_and_intensity: [f32; 4],
}
