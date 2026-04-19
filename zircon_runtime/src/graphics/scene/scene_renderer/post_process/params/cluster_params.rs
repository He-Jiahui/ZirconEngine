use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ClusterParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_clusters: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) counts: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) strengths: [f32; 4],
}
