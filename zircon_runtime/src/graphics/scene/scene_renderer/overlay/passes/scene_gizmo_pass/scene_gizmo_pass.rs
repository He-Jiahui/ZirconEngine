use crate::graphics::scene::scene_renderer::overlay::ViewportIconAtlas;

/// 场景实体辅助图形的资源持有者；同时管理图标上传生命周期和无图标时的线形回退。
pub(crate) struct SceneGizmoPass {
    pub(super) icon_pipeline: wgpu::RenderPipeline,
    pub(super) icon_sampler: wgpu::Sampler,
    pub(super) icon_atlas: ViewportIconAtlas,
}
