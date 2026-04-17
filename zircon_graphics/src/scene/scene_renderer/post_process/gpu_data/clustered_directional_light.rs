use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::post_process) struct ClusteredDirectionalLight {
    pub(in crate::scene::scene_renderer::post_process) direction: [f32; 4],
    pub(in crate::scene::scene_renderer::post_process) color_intensity: [f32; 4],
}
