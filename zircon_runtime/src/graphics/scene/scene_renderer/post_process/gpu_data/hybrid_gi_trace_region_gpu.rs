use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct GpuHybridGiTraceRegion {
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) boost_and_coverage: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) rt_lighting_rgb_and_weight: [f32; 4],
}
