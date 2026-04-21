use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{build_mesh_draws, MeshDraw};
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::scene_renderer_core::SceneRendererCore;

pub(super) struct CompiledSceneDraws {
    pub(super) draws: Vec<MeshDraw>,
    pub(super) indirect_segment_count: u32,
    pub(super) indirect_args_count: u32,
    pub(super) indirect_args_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(super) indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(super) indirect_authority_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(super) indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(super) indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

pub(super) fn build_compiled_scene_draws(
    renderer: &SceneRendererCore,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
) -> CompiledSceneDraws {
    let built_mesh_draws = build_mesh_draws(
        device,
        encoder,
        &renderer.virtual_geometry_indirect_args,
        &renderer.model_bind_group_layout,
        streamer,
        frame,
        virtual_geometry_enabled,
    );

    CompiledSceneDraws {
        draws: built_mesh_draws.draws,
        indirect_segment_count: built_mesh_draws.indirect_segment_count,
        indirect_args_count: built_mesh_draws.indirect_args_count,
        indirect_args_buffer: built_mesh_draws.indirect_args_buffer,
        indirect_submission_buffer: built_mesh_draws.indirect_submission_buffer,
        indirect_authority_buffer: built_mesh_draws.indirect_authority_buffer,
        indirect_draw_ref_buffer: built_mesh_draws.indirect_draw_ref_buffer,
        indirect_segment_buffer: built_mesh_draws.indirect_segment_buffer,
    }
}
