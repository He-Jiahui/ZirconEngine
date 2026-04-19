use std::sync::Arc;

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::hybrid_gi::HybridGiGpuResources;
use super::super::super::super::mesh::{
    MeshPipelineCache, VirtualGeometryIndirectArgsGpuResources,
};
use super::super::super::super::overlay::{
    ViewportIconSource, ViewportOverlayRenderer,
};
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::ScenePostProcessResources;
use super::super::super::super::prepass::NormalPrepassPipeline;
use super::super::super::super::ui::ScreenSpaceUiRenderer;
use super::super::super::super::virtual_geometry::VirtualGeometryGpuResources;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::layouts::{create_model_bind_group_layout, create_texture_bind_group_layout};
use super::super::scene_bind_group_bundle::create_scene_bind_group_bundle;

impl SceneRendererCore {
    pub(crate) fn new_with_icon_source(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        icon_source: Arc<dyn ViewportIconSource>,
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
        let screen_space_ui_renderer = ScreenSpaceUiRenderer::new(device, target_format);
        let hybrid_gi = HybridGiGpuResources::new(device);
        let virtual_geometry = VirtualGeometryGpuResources::new(device);
        let virtual_geometry_indirect_args = VirtualGeometryIndirectArgsGpuResources::new(device);

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
            hybrid_gi,
            virtual_geometry,
            virtual_geometry_indirect_args,
        }
    }
}
