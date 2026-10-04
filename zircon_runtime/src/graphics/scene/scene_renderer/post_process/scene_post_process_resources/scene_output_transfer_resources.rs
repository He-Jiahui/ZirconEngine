use super::super::resources::terminal_resource_cache::TerminalPostProcessResourceCache;

/// 只执行最终输出转换的启动资源，保留终端区域缓存而省去完整效果图资源。
/// 由精简启动配置构造，执行时经资源枚举的输出转换分支访问。
pub(crate) struct SceneOutputTransferResources {
    pub(in crate::graphics::scene::scene_renderer::post_process) terminal_resource_cache:
        TerminalPostProcessResourceCache,
    pub(in crate::graphics::scene::scene_renderer::post_process) bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) pipeline: wgpu::RenderPipeline,
}
