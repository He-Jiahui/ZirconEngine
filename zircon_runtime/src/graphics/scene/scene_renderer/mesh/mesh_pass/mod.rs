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
    INDIRECT_COMPACTION_METADATA_STRIDE_BYTES, INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX,
    INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
    IndirectCompactionBatchMetadata, IndirectCompactionBatchRange, IndirectCompactionPlan,
};
pub(crate) use indirect_compaction_resources::MeshIndirectCompactionResources;
pub(crate) use indirect_draw_batcher::{
    IndirectDrawBatch, IndirectDrawBatcher, IndirectDrawBatcherStats,
};
pub(crate) use indirect_draw_execution::{
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, MeshDrawCommandStream, MeshIndirectArgsReadback,
    MeshIndirectArgsSnapshot, MeshIndirectDrawExecution, MeshPassIndirectDrawExecutions,
};
pub(crate) use mesh_draw_command::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
    MeshPassPipelineKind, MeshPipelineVariantId,
};
pub(crate) use mesh_draw_command_list::{
    MeshDrawCommandList, MeshDrawCommandListStats, MeshPassCommandBufferStats,
    MeshPassCommandBuffers, build_mesh_pass_command_buffers,
    build_mesh_pass_command_buffers_cached,
};
pub(crate) use mesh_pass_processor::{
    MeshBatchCacheIdentity, MeshBatchRef, MeshPassBuildContext, MeshPassProcessor,
};
pub(crate) use processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
pub(crate) use replay::{
    GPU_SCENE_BIND_GROUP_SLOT, MeshDrawCommandReplayer, MeshDrawReplayStats,
    MeshDrawReplayStatsAccumulator, MeshSceneDataBindHandle,
};
