mod build_mesh_draws;
mod mesh_draw;
pub(crate) mod mesh_pass;
mod mesh_pipeline;
mod mesh_pipeline_cache;
mod prepared_queue;
pub(in crate::graphics::scene) mod skinning;

pub(crate) use build_mesh_draws::{build_mesh_draws, BuiltMeshDraws, IndexedIndirectArgs};
pub(crate) use mesh_draw::MeshDraw;
pub(crate) use mesh_pass::{
    build_mesh_pass_command_buffers, build_mesh_pass_command_buffers_cached,
    CachedMeshDrawCommands, MeshDrawReplayStatsAccumulator, MeshIndirectArgsReadback,
    MeshPassIndirectDrawExecutions,
};
pub(crate) use mesh_pipeline::FALLBACK_MESH_SHADER;
pub(crate) use mesh_pipeline_cache::MeshPipelineCache;
pub(crate) use prepared_queue::{prepare_mesh_queue, PreparedMeshQueueStats};
