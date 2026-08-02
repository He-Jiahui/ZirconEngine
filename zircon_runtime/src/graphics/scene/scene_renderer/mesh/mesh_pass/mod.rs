mod cached_mesh_draw_commands;
mod indirect_compaction;
mod indirect_compaction_resources;
mod indirect_draw_batcher;
mod indirect_draw_execution;
mod mesh_draw_command;
mod mesh_draw_command_list;
mod mesh_pass_processor;
mod processors;
mod replay;

pub(crate) use cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
pub(crate) use indirect_compaction::{
    IndirectCompactionBatchMetadata, IndirectCompactionBatchRange, IndirectCompactionPlan,
    INDIRECT_COMPACTION_METADATA_STRIDE_BYTES, INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX,
    INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
};
pub(crate) use indirect_compaction_resources::MeshIndirectCompactionResources;
pub(crate) use indirect_draw_batcher::{
    IndirectDrawBatch, IndirectDrawBatcher, IndirectDrawBatcherStats,
};
pub(crate) use indirect_draw_execution::{
    MeshDrawCommandStream, MeshIndirectArgsReadback, MeshIndirectArgsSnapshot,
    MeshIndirectDrawExecution, MeshPassIndirectDrawExecutions, INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
};
pub(crate) use mesh_draw_command::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
    MeshPassPipelineKind, MeshPipelineVariantId,
};
pub(crate) use mesh_draw_command_list::{
    build_mesh_pass_command_buffers, build_mesh_pass_command_buffers_cached,
    build_mesh_pass_command_buffers_cached_parallel, MeshDrawCommandList, MeshDrawCommandListStats,
    MeshPassCommandBufferStats, MeshPassCommandBuffers,
};
pub(crate) use mesh_pass_processor::{
    depth_prepass_command_spec, mesh_pass_command_specs, opaque_base_command_spec,
    shadow_command_spec, taa_reactive_mask_command_spec, transparent_command_spec,
    velocity_command_spec, MeshBatchCacheIdentity, MeshBatchRef, MeshPassBuildContext,
    MeshPassCommandSpec, MeshPassProcessor,
};
pub(crate) use processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
pub(crate) use replay::{
    MeshDrawCommandReplayer, MeshDrawReplayStats, MeshDrawReplayStatsAccumulator,
    MeshSceneDataBindHandle, GPU_SCENE_BIND_GROUP_SLOT,
};
