use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{BuiltMeshDraws, MeshDraw};
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::scene_renderer_core::SceneRendererAdvancedPluginResources;

pub(super) struct CompiledSceneDraws {
    draws: Vec<MeshDraw>,
    indirect_segment_count: u32,
    indirect_args_count: u32,
    indirect_args_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

impl CompiledSceneDraws {
    fn from_built_mesh_draws(built_mesh_draws: BuiltMeshDraws) -> Self {
        let indirect_segment_count = built_mesh_draws.indirect_segment_count();
        let indirect_args_count = built_mesh_draws.indirect_args_count();
        let indirect_args_buffer = built_mesh_draws.indirect_args_buffer();
        let indirect_submission_buffer = built_mesh_draws.indirect_submission_buffer();
        let indirect_authority_buffer = built_mesh_draws.indirect_authority_buffer();
        let indirect_draw_ref_buffer = built_mesh_draws.indirect_draw_ref_buffer();
        let indirect_segment_buffer = built_mesh_draws.indirect_segment_buffer();
        Self {
            draws: built_mesh_draws.into_draws(),
            indirect_segment_count,
            indirect_args_count,
            indirect_args_buffer,
            indirect_submission_buffer,
            indirect_authority_buffer,
            indirect_draw_ref_buffer,
            indirect_segment_buffer,
        }
    }

    pub(super) fn draws(&self) -> &[MeshDraw] {
        &self.draws
    }

    pub(super) fn draws_mut(&mut self) -> &mut [MeshDraw] {
        &mut self.draws
    }

    pub(super) fn indirect_segment_count(&self) -> u32 {
        self.indirect_segment_count
    }

    pub(super) fn indirect_args_count(&self) -> u32 {
        self.indirect_args_count
    }

    pub(super) fn indirect_args_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.clone()
    }

    pub(super) fn indirect_submission_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_submission_buffer.clone()
    }

    pub(super) fn indirect_authority_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_authority_buffer.clone()
    }

    pub(super) fn indirect_draw_ref_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_draw_ref_buffer.clone()
    }

    pub(super) fn indirect_segment_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_segment_buffer.clone()
    }
}

pub(super) fn build_compiled_scene_draws(
    advanced_plugin_resources: &SceneRendererAdvancedPluginResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    model_bind_group_layout: &wgpu::BindGroupLayout,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
) -> CompiledSceneDraws {
    let built_mesh_draws = advanced_plugin_resources.build_mesh_draws(
        device,
        encoder,
        model_bind_group_layout,
        streamer,
        frame,
        virtual_geometry_enabled,
    );

    CompiledSceneDraws::from_built_mesh_draws(built_mesh_draws)
}
