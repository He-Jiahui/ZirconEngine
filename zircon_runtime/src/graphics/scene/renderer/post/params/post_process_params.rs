use bytemuck::{Pod, Zeroable};

/// 后处理共用的帧参数快照，合并逻辑视口、编译后的功能开关和本帧可用资源计数。
/// 由参数构建入口统一编码，并与 `post_process.wgsl` 的 uniform 字段顺序保持一致。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct PostProcessParams {
    /// xy 为本阶段局部尺寸，zw 为场景几何和历史纹理的物理视口原点。
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_clusters: [u32; 4],
    /// xy 为二维聚合网格，zw 为当前颜色输入原点；颜色已是局部目标时可与物理原点不同。
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_dimensions: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) feature_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) lighting_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_counts: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_source_ledger: [u32; 4],
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

/// 终端抗锯齿和输出转换使用的目标区域原点，区分局部中间纹理与物理视口。
/// 由终端资源缓存按原点保存不可变缓冲，避免多视口共用一次可变上传。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct TerminalRegionParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_origin: [u32; 4],
}
