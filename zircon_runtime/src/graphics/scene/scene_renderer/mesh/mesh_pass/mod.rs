#![allow(dead_code, unused_imports)]

mod cached_mesh_draw_commands;
mod indirect_draw_batcher;
mod indirect_draw_execution;
mod mesh_draw_command;
mod mesh_draw_command_list;
mod mesh_pass_processor;
mod processors;
mod replay;

pub(crate) use cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, MeshDrawCommandCacheStats,
};
pub(crate) use indirect_draw_batcher::{
    IndirectDrawBatch, IndirectDrawBatcher, IndirectDrawBatcherStats,
};
pub(crate) use indirect_draw_execution::{
    MeshDrawCommandStream, MeshIndirectDrawExecution, MeshPassIndirectDrawExecutions,
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
};
pub(crate) use mesh_draw_command::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
    MeshPassPipelineKind, MeshPipelineVariantId,
};
pub(crate) use mesh_draw_command_list::{
    build_mesh_pass_command_buffers, build_mesh_pass_command_buffers_cached, MeshDrawCommandList,
    MeshDrawCommandListStats, MeshPassCommandBufferStats, MeshPassCommandBuffers,
};
pub(crate) use mesh_pass_processor::{
    MeshBatchCacheIdentity, MeshBatchRef, MeshPassBuildContext, MeshPassProcessor,
};
pub(crate) use processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor, TransparentPassProcessor,
    VelocityPassProcessor,
};
pub(crate) use replay::{
    MeshDrawCommandReplayer, MeshDrawReplayStats, MeshDrawReplayStatsAccumulator,
    MeshSceneDataBindHandle, GPU_SCENE_BIND_GROUP_SLOT,
};
