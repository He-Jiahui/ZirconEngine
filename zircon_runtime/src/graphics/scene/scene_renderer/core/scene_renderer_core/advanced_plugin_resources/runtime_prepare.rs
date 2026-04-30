use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::graphics::hybrid_gi_extract_sources::enabled_hybrid_gi_extract;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        let hybrid_gi_extract =
            enabled_hybrid_gi_extract(frame.extract.lighting.hybrid_global_illumination.as_ref());
        let hybrid_gi_gpu_readback = if let Some(hybrid_gi) = self.hybrid_gi() {
            hybrid_gi.execute_prepare(
                device,
                queue,
                encoder,
                streamer,
                frame.hybrid_gi_prepare.as_ref(),
                frame.hybrid_gi_scene_prepare.as_ref(),
                frame.hybrid_gi_resolve_runtime.as_ref(),
                hybrid_gi_extract,
                &frame.extract.geometry.meshes,
                &frame.extract.lighting.directional_lights,
                &frame.extract.lighting.point_lights,
                &frame.extract.lighting.spot_lights,
                hybrid_gi_extract.map(|hybrid_gi| hybrid_gi.probe_budget),
                hybrid_gi_extract.map(|hybrid_gi| hybrid_gi.tracing_budget),
            )?
        } else {
            None
        };
        let virtual_geometry_gpu_readback = if let Some(virtual_geometry) = self.virtual_geometry()
        {
            virtual_geometry.execute_prepare(
                device,
                queue,
                encoder,
                frame.virtual_geometry_prepare.as_ref(),
                frame
                    .extract
                    .geometry
                    .virtual_geometry
                    .as_ref()
                    .map(|virtual_geometry| virtual_geometry.page_budget),
            )?
        } else {
            None
        };

        Ok(SceneRendererAdvancedPluginReadbacks::new(
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
        ))
    }
}
