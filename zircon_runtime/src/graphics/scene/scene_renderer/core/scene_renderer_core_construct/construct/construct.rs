use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::{
    GraphicsError, RenderFeatureDescriptor, RuntimePrepareCollectorRegistration,
};

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::hzb::{hzb_occlusion_supported_by_limits, HzbOcclusionCuller};
use super::super::super::super::mesh::skinning::{
    create_empty_skinned_joint_palette_buffer, skinned_joint_palette_uniform_min_binding_size,
};
use super::super::super::super::mesh::{CachedMeshDrawCommands, MeshPipelineCache};
use super::super::super::super::overlay::{ViewportIconSource, ViewportOverlayRenderer};
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::ScenePostProcessResources;
use super::super::super::super::scene_clear::SceneRegionClearResources;
use super::super::super::super::shadow::atlas::{
    ShadowAtlasAllocator, ShadowAtlasConfig, ShadowAtlasResourceConfig, ShadowAtlasResources,
    SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT,
};
use super::super::super::super::shadow::ShadowMapRenderer;
use super::super::super::super::sprite::SpriteRenderer;
use super::super::super::super::ui::ScreenSpaceUiRenderer;
use super::super::super::constants::DEPTH_FORMAT;
use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginResources, SceneRendererCore,
};
use super::super::layouts::{
    create_material_texture_bind_group_layout, create_texture_bind_group_layout,
};
use super::super::scene_bind_group_bundle::create_scene_bind_group_bundle;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        backend_name: &str,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: &[RenderFeatureDescriptor],
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Result<Self, GraphicsError> {
        let plugin_shading_models = plugin_shading_models.into_iter().collect::<Vec<_>>();
        let scene_bind_group_bundle = create_scene_bind_group_bundle(device);
        let skinned_joint_palette_fallback_buffer =
            create_empty_skinned_joint_palette_buffer(device);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);
        let material_texture_bind_group_layout = create_material_texture_bind_group_layout(device);
        let gpu_scene = GpuScene::new(
            device,
            Arc::clone(&skinned_joint_palette_fallback_buffer),
            skinned_joint_palette_uniform_min_binding_size(),
        );

        let mut mesh_pipelines = MeshPipelineCache::new(
            device,
            target_format,
            &scene_bind_group_bundle.layout,
            &material_texture_bind_group_layout,
            gpu_scene.scene_bind_group_layout(),
        );
        for descriptor in plugin_geometry_sources {
            mesh_pipelines.register_geometry_source_descriptor(descriptor);
        }
        let scene_clear = SceneRegionClearResources::new(device, target_format, DEPTH_FORMAT);
        let shadow_map_renderer = ShadowMapRenderer::new(device, &scene_bind_group_bundle.layout);
        let shadow_atlas_resources =
            ShadowAtlasResources::new(device, ShadowAtlasResourceConfig::default());
        let shadow_atlas_resource_config = shadow_atlas_resources.config();
        let shadow_atlas_allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig {
            width: shadow_atlas_resource_config.width,
            height: shadow_atlas_resource_config.height,
            reserved_top_px: SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT
                .min(shadow_atlas_resource_config.height),
            ..ShadowAtlasConfig::default()
        });
        let deferred = DeferredSceneResources::new(
            device,
            asset_manager.as_ref(),
            &scene_bind_group_bundle.layout,
            &material_texture_bind_group_layout,
            gpu_scene.scene_bind_group_layout(),
            target_format,
            &plugin_shading_models,
        )?;
        let particle_renderer =
            ParticleRenderer::new(device, &scene_bind_group_bundle.layout, target_format);
        let sprite_renderer = SpriteRenderer::new(
            device,
            &scene_bind_group_bundle.layout,
            &texture_bind_group_layout,
            target_format,
        );
        let hzb_occlusion_culler = hzb_occlusion_supported_by_limits(&device.limits()).then(|| {
            HzbOcclusionCuller::new(
                device,
                &scene_bind_group_bundle.layout,
                gpu_scene.scene_bind_group_layout(),
            )
        });
        let post_process =
            ScenePostProcessResources::new(device, queue, target_format, backend_name);
        let overlay_renderer = ViewportOverlayRenderer::new(
            device,
            target_format,
            &scene_bind_group_bundle.layout,
            &texture_bind_group_layout,
            icon_source,
        );
        let screen_space_ui_renderer =
            ScreenSpaceUiRenderer::new(asset_manager, device, queue, target_format);
        let advanced_plugin_resources = SceneRendererAdvancedPluginResources::new(
            device,
            render_features,
            runtime_prepare_collectors,
        );

        Ok(Self {
            texture_bind_group_layout,
            scene_bind_group_layout: scene_bind_group_bundle.layout,
            scene_uniform_buffer: scene_bind_group_bundle.uniform_buffer,
            scene_environment_sample_buffer: scene_bind_group_bundle.environment_sample_buffer,
            scene_bind_group: scene_bind_group_bundle.bind_group,
            target_format,
            depth_format: DEPTH_FORMAT,
            mesh_command_generation: 0,
            material_texture_bind_group_layout,
            mesh_pipelines,
            cached_mesh_draw_commands: CachedMeshDrawCommands::default(),
            gpu_scene,
            hzb_occlusion_culler,
            scene_clear,
            shadow_map_renderer,
            shadow_atlas_allocator,
            shadow_atlas_resources,
            deferred,
            particle_renderer,
            sprite_renderer,
            post_process,
            overlay_renderer,
            screen_space_ui_renderer,
            transient_resource_pool: Default::default(),
            advanced_plugin_resources,
        })
    }
}
