use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct PostProcessParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_clusters: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) feature_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_counts: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) blends: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) grading: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tint_and_probe: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_color_and_intensity:
        [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) baked_color_and_intensity:
        [f32; 4],
}
