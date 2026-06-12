use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct HzbParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) target_size: [u32; 2],
    pub(in crate::graphics::scene::scene_renderer::post_process) target_mip_level: u32,
    pub(in crate::graphics::scene::scene_renderer::post_process) _pad0: u32,
}
