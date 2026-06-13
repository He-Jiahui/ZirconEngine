use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct PostProcessParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_clusters: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) feature_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) lighting_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_counts: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) anti_alias: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) blends: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) grading: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tint_and_probe: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_color_and_intensity:
        [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) baked_color_and_intensity:
        [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_tonemap_lut: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_blur_dof: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_dof_lens: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_vignette_grain: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_chromatic_fog: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_fog_color: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_dither_ssr: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_ssr_limits: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_depth: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_projection: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_view_x: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_view_y: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_view_z: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_motion_blur: [f32; 4],
}
