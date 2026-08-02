mod build_mesh_draws;
mod mesh_draw;
pub(crate) mod mesh_pass;
mod mesh_pipeline;
mod mesh_pipeline_cache;
mod prepared_queue;
pub(in crate::graphics::scene) mod skinning;

pub(crate) use build_mesh_draws::{
    build_mesh_draws, BuiltMeshDraws, IndexedIndirectArgs,
    PendingMeshCommandCacheExtractionContext, PendingMeshCommandCacheExtractionStats,
    PendingMeshCommandCachePlanStats,
};
pub(crate) use mesh_draw::MeshDraw;
pub(crate) use mesh_pass::{
    build_mesh_pass_command_buffers, build_mesh_pass_command_buffers_cached,
    build_mesh_pass_command_buffers_cached_parallel, CachedMeshDrawCommands,
    MeshDrawReplayStatsAccumulator, MeshIndirectArgsReadback, MeshPassCommandBuffers,
    MeshPassIndirectDrawExecutions,
};
pub(crate) use mesh_pipeline::FALLBACK_MESH_SHADER;
pub(crate) use mesh_pipeline_cache::{
    create_mesh_prewarm_validation_pipeline_layout,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    validate_mesh_prewarm_request_render_pipeline, EnvironmentOnlyPbrBasePipelinePrewarmReport,
    MeshPipelineCache, MeshPipelineShaderSource,
};
pub use mesh_pipeline_cache::{
    RuntimeShaderPipelinePrewarmFailure, RuntimeShaderPipelinePrewarmReport,
};
pub(crate) use prepared_queue::{
    prepare_mesh_queue, PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats,
};
