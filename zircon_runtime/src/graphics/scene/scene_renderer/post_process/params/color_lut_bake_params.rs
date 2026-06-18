use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ColorLutBakeParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) lut_size_and_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tonemap_lut: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) grading: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tint_and_exposure: [f32; 4],
}
