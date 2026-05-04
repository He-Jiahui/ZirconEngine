use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::graphics::{RenderFeatureDescriptor, RuntimePrepareCollectorRegistration};

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::mesh::MeshPipelineCache;
use super::super::super::super::overlay::{ViewportIconSource, ViewportOverlayRenderer};
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::ScenePostProcessResources;
use super::super::super::super::prepass::NormalPrepassPipeline;
use super::super::super::super::ui::ScreenSpaceUiRenderer;
use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginResources, SceneRendererCore,
};
use super::super::layouts::{create_model_bind_group_layout, create_texture_bind_group_layout};
use super::super::scene_bind_group_bundle::create_scene_bind_group_bundle;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: &[RenderFeatureDescriptor],
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Self {
        let scene_bind_group_bundle = create_scene_bind_group_bundle(device);
        let model_bind_group_layout = create_model_bind_group_layout(device);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);

        let mesh_pipelines = MeshPipelineCache::new(
            device,
            target_format,
            &scene_bind_group_bundle.layout,
            &model_bind_group_layout,
            &texture_bind_group_layout,
        );
        let normal_prepass = NormalPrepassPipeline::new(
            device,
            &scene_bind_group_bundle.layout,
            &model_bind_group_layout,
        );
        let deferred = DeferredSceneResources::new(
            device,
            &scene_bind_group_bundle.layout,
            &model_bind_group_layout,
            &texture_bind_group_layout,
            target_format,
        );
        let particle_renderer =
            ParticleRenderer::new(device, &scene_bind_group_bundle.layout, target_format);
        let post_process = ScenePostProcessResources::new(device, queue, target_format);
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

        Self {
            texture_bind_group_layout,
            scene_uniform_buffer: scene_bind_group_bundle.uniform_buffer,
            scene_bind_group: scene_bind_group_bundle.bind_group,
            model_bind_group_layout,
            mesh_pipelines,
            normal_prepass,
            deferred,
            particle_renderer,
            post_process,
            overlay_renderer,
            screen_space_ui_renderer,
            advanced_plugin_resources,
        }
    }
}
