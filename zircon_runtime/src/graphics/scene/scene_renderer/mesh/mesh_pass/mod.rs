mod cached_mesh_draw_commands;
mod indirect_buffer_upload;
mod indirect_compaction;
mod indirect_compaction_resources;
mod indirect_draw_batcher;
mod indirect_draw_execution;
mod indirect_draw_plan;
mod indirect_draw_workspace;
mod mesh_draw_command;
mod mesh_draw_command_list;
mod mesh_pass_processor;
mod pipeline_variant_pin_counts;
mod processors;
mod replay;

pub(crate) use cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
use indirect_buffer_upload::{PodRangeUploadCommit, PodRangeUploadShadow};
pub(crate) use indirect_compaction::{
    INDIRECT_COMPACTION_METADATA_STRIDE_BYTES, INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX,
    INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
    IndirectCompactionBatchMetadata, IndirectCompactionBatchRange, IndirectCompactionPlan,
};
use indirect_compaction_resources::grow_indirect_buffer_capacity;
pub(crate) use indirect_compaction_resources::{
    MeshIndirectCompactionResources, MeshIndirectCompactionWorkspace,
};
pub(crate) use indirect_draw_batcher::{
    IndirectDrawBatch, IndirectDrawBatcher, IndirectDrawBatcherStats,
};
pub(crate) use indirect_draw_execution::{
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, MeshDrawCommandStream, MeshIndirectArgsReadback,
    MeshIndirectArgsSnapshot, MeshIndirectDrawExecution, MeshIndirectResourceIdentity,
    MeshPassIndirectDrawExecutions,
};
pub(crate) use indirect_draw_plan::{MeshIndirectDrawPlan, MeshPassIndirectDrawPlans};
pub(crate) use indirect_draw_workspace::{
    MeshIndirectDrawWorkspace, MeshIndirectPhaseWorkspace, MeshIndirectWorkspaceFrameStats,
    MeshIndirectWorkspacePreparedUpload,
};
pub(crate) use mesh_draw_command::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandPayload,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};
pub(crate) use mesh_draw_command_list::{
    MeshDrawCommandList, MeshDrawCommandListStats, MeshPassCommandBufferStats,
    MeshPassCommandBuffers, build_environment_capture_command_buffers,
    build_hit_proxy_command_list, build_mesh_pass_command_buffers,
    build_mesh_pass_command_buffers_cached, build_mesh_pass_command_buffers_cached_parallel,
};
pub(crate) use mesh_pass_processor::{
    MeshBatchCacheIdentity, MeshBatchRef, MeshPassBuildContext, MeshPassCommandSpec,
    MeshPassProcessor, depth_prepass_command_spec, hit_proxy_command_spec, mesh_pass_command_specs,
    opaque_base_command_spec, shadow_command_spec, taa_reactive_mask_command_spec,
    transparent_command_spec, velocity_command_spec,
};
pub(crate) use processors::{
    DepthPrepassProcessor, HitProxyPassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
pub(crate) use replay::{
    GPU_SCENE_BIND_GROUP_SLOT, MeshDrawCommandReplayer, MeshDrawReplayStats,
    MeshDrawReplayStatsAccumulator, MeshSceneDataBindHandle,
};
