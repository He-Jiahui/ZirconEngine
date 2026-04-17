use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::post_process) struct BloomParams {
    pub(in crate::scene::scene_renderer::post_process) viewport: [u32; 4],
    pub(in crate::scene::scene_renderer::post_process) tuning: [f32; 4],
}
