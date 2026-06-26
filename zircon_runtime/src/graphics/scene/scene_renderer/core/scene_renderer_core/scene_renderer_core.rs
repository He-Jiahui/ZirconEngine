use crate::graphics::scene::gpu_scene::GpuScene;

use super::super::super::deferred::DeferredSceneResources;
use super::super::super::graph_execution::TransientResourcePool;
use super::super::super::hzb::HzbOcclusionCuller;
use super::super::super::mesh::{CachedMeshDrawCommands, MeshPipelineCache};
use super::super::super::overlay::ViewportOverlayRenderer;
use super::super::super::particle::ParticleRenderer;
use super::super::super::post_process::ScenePostProcessResources;
use super::super::super::scene_clear::SceneRegionClearResources;
use super::super::super::shadow::atlas::{ShadowAtlasAllocator, ShadowAtlasResources};
use super::super::super::shadow::ShadowMapRenderer;
use super::super::super::sprite::SpriteRenderer;
use super::super::super::ui::ScreenSpaceUiRenderer;
use super::SceneRendererAdvancedPluginResources;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) texture_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) scene_uniform_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core) scene_bind_group: wgpu::BindGroup,
    pub(in crate::graphics::scene::scene_renderer::core) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core) depth_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_command_generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) material_texture_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_pipelines: MeshPipelineCache,
    pub(in crate::graphics::scene::scene_renderer::core) cached_mesh_draw_commands:
        CachedMeshDrawCommands,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_scene: GpuScene,
    pub(in crate::graphics::scene::scene_renderer::core) hzb_occlusion_culler:
        Option<HzbOcclusionCuller>,
    pub(in crate::graphics::scene::scene_renderer::core) scene_clear: SceneRegionClearResources,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_map_renderer: ShadowMapRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_atlas_allocator:
        ShadowAtlasAllocator,
    pub(in crate::graphics::scene::scene_renderer::core) shadow_atlas_resources:
        ShadowAtlasResources,
    pub(in crate::graphics::scene::scene_renderer::core) deferred: DeferredSceneResources,
    pub(in crate::graphics::scene::scene_renderer::core) particle_renderer: ParticleRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) sprite_renderer: SpriteRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) post_process: ScenePostProcessResources,
    pub(in crate::graphics::scene::scene_renderer::core) overlay_renderer: ViewportOverlayRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) screen_space_ui_renderer:
        ScreenSpaceUiRenderer,
    pub(in crate::graphics::scene::scene_renderer::core) transient_resource_pool:
        TransientResourcePool,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_resources:
        SceneRendererAdvancedPluginResources,
}
