use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::post_process) struct GpuHybridGiProbe {
    pub(in crate::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::scene::scene_renderer::post_process) irradiance_and_intensity: [f32; 4],
    pub(in crate::scene::scene_renderer::post_process) hierarchy_rt_lighting_rgb_and_weight:
        [f32; 4],
}
