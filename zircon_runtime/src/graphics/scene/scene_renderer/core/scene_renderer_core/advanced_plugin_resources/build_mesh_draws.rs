use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{build_mesh_draws, BuiltMeshDraws};
use crate::graphics::types::ViewportRenderFrame;

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        model_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
    ) -> BuiltMeshDraws {
        build_mesh_draws(
            device,
            encoder,
            self.virtual_geometry_indirect_args(),
            model_layout,
            streamer,
            frame,
            virtual_geometry_enabled,
        )
    }
}
