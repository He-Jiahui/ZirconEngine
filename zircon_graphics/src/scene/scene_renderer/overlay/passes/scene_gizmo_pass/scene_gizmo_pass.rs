use crate::scene::scene_renderer::overlay::ViewportIconAtlas;

pub(crate) struct SceneGizmoPass {
    pub(super) icon_pipeline: wgpu::RenderPipeline,
    pub(super) icon_sampler: wgpu::Sampler,
    pub(super) icon_atlas: ViewportIconAtlas,
}
