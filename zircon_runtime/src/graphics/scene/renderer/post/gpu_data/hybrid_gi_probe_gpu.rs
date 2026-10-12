use bytemuck::{Pod, Zeroable};

/// 投影到当前视口的混合 GI 探针贡献，供后处理合成与历史可信度判断使用。
/// 这是 `post_process.wgsl` 的存储缓冲布局，不能替代世界空间探针或跨视口复用投影结果。
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct GpuHybridGiProbe {
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) irradiance_and_intensity: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) hierarchy_irradiance_rgb_and_weight:
        [f32; 4],
    /// 依次承载历史签名、场景可信度、来源位掩码和动态权重，着色器会读取全部四项。
    pub(in crate::graphics::scene::scene_renderer::post_process) hierarchy_rt_lighting_rgb_and_weight:
        [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) temporal_signature_and_padding:
        [f32; 4],
}
