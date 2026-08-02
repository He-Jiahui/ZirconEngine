use super::super::resources::terminal_resource_cache::TerminalPostProcessResourceCache;

pub(crate) struct SceneOutputTransferResources {
    pub(in crate::graphics::scene::scene_renderer::post_process) terminal_resource_cache:
        TerminalPostProcessResourceCache,
    pub(in crate::graphics::scene::scene_renderer::post_process) bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) pipeline: wgpu::RenderPipeline,
}
