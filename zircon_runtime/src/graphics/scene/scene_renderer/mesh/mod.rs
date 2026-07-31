mod build_mesh_draws;
mod mesh_draw;
pub(crate) mod mesh_pass;
mod mesh_pipeline;
mod mesh_pipeline_cache;
mod prepared_queue;
pub(in crate::graphics::scene) mod skinning;

pub(crate) use build_mesh_draws::{
    BuiltMeshDraws, IndexedIndirectArgs, PendingMeshCommandCacheExtractionContext,
    PendingMeshCommandCacheExtractionStats, PendingMeshCommandCachePlanStats, build_mesh_draws,
};
pub(crate) use mesh_draw::MeshDraw;
pub(crate) use mesh_pass::{
    CachedMeshDrawCommands, MeshDrawReplayStatsAccumulator, MeshIndirectArgsReadback,
    MeshPassCommandBuffers, MeshPassIndirectDrawExecutions, build_mesh_pass_command_buffers,
    build_mesh_pass_command_buffers_cached,
};
pub(crate) use mesh_pipeline::FALLBACK_MESH_SHADER;
pub(crate) use mesh_pipeline_cache::{
    EnvironmentOnlyPbrBasePipelinePrewarmReport, MeshPipelineCache, MeshPipelineShaderSource,
    create_mesh_prewarm_validation_pipeline_layout,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    validate_mesh_prewarm_request_render_pipeline,
};
pub(crate) use prepared_queue::{
    PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats, prepare_mesh_queue,
};
