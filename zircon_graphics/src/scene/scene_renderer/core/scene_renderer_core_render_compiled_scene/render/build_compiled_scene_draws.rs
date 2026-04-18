use crate::scene::resources::ResourceStreamer;
use crate::scene::scene_renderer::mesh::{build_mesh_draws, MeshDraw};
use crate::types::EditorOrRuntimeFrame;

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::virtual_geometry_indirect_stats::{
    virtual_geometry_indirect_stats, VirtualGeometryIndirectStats,
};

pub(super) struct CompiledSceneDraws {
    pub(super) draws: Vec<MeshDraw>,
    pub(super) indirect_stats: VirtualGeometryIndirectStats,
}

pub(super) fn build_compiled_scene_draws(
    renderer: &SceneRendererCore,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &ResourceStreamer,
    frame: &EditorOrRuntimeFrame,
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
    let indirect_stats = virtual_geometry_indirect_stats(
        &built_mesh_draws.draws,
        built_mesh_draws.indirect_args_count,
        built_mesh_draws.indirect_segment_count,
        built_mesh_draws.indirect_draw_ref_buffer,
        built_mesh_draws.indirect_segment_buffer,
    );

    CompiledSceneDraws {
        draws: built_mesh_draws.draws,
        indirect_stats,
    }
}
