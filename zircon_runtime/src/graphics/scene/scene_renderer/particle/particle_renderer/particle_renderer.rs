/// 生命周期随场景 renderer 的辅助效果配置；保存深度测试颜色、HUD 覆盖颜色和速度三套管线。
/// 顶点是本帧临时资源；场景参数、目标纹理与 render region 均由 graph 执行上下文提供。
pub(crate) struct ParticleRenderer {
    pub(in crate::graphics::scene::scene_renderer::particle) pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::particle) overlay_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::particle) velocity_pipeline:
        wgpu::RenderPipeline,
}
