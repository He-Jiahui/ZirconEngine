use bytemuck::{Pod, Zeroable};

/// 将本帧调度的 GI 追踪区域投影为后处理可消费的覆盖与光照贡献。
/// 有效前缀及其计数由编码器一起产生，字段顺序须与 WGSL 存储布局保持一致。
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct GpuHybridGiTraceRegion {
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) boost_and_coverage: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) rt_lighting_rgb_and_weight:
        [f32; 4],
}
