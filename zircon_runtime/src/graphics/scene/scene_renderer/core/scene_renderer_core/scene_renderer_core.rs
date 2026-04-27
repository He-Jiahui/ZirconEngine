use super::super::super::deferred::DeferredSceneResources;
use super::super::super::mesh::MeshPipelineCache;
use super::super::super::overlay::ViewportOverlayRenderer;
use super::super::super::particle::ParticleRenderer;
use super::super::super::post_process::ScenePostProcessResources;
use super::super::super::prepass::NormalPrepassPipeline;
use super::super::super::ui::ScreenSpaceUiRenderer;
use super::SceneRendererAdvancedPluginResources;

pub(crate) struct SceneRendererCore {
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) scene_uniform_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group: wgpu::BindGroup,
    pub(in crate::graphics::scene::scene_renderer::core) model_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_pipelines: MeshPipelineCache,
    pub(in crate::graphics::scene::scene_renderer::core) normal_prepass: NormalPrepassPipeline,
    pub(in crate::graphics::scene::scene_renderer::core) deferred: DeferredSceneResources,
    pub(in crate::graphics::scene::scene_renderer::core) particle_renderer: ParticleRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) post_process: ScenePostProcessResources,
    pub(in crate::graphics::scene::scene_renderer::core) overlay_renderer: ViewportOverlayRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) screen_space_ui_renderer:
        ScreenSpaceUiRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_resources:
        SceneRendererAdvancedPluginResources,
}
