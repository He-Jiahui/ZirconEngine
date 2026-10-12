#[path = "build/mod.rs"]
mod build;
#[path = "../../../mesh/build_mesh_draws/create_mesh_draw.rs"]
mod create_mesh_draw;
#[path = "../../../mesh/build_mesh_draws/indexed_indirect_args.rs"]
mod indexed_indirect_args;
#[path = "../../../mesh/build_mesh_draws/raster_draws_for_mesh.rs"]
mod raster_draws_for_mesh;

pub(crate) use build::{
    build_mesh_draws, BuiltMeshDraws, MaterialPipelineFeatureSet,
    MaterialPipelineRequirementCensus, PendingMeshCommandCacheExtractionContext,
    PendingMeshCommandCacheExtractionStats, PendingMeshCommandCachePlanStats,
};
pub(crate) use indexed_indirect_args::IndexedIndirectArgs;

/// 按稳定实例键提供当前拾取快照的非零令牌；hit-proxy 构建会过滤 None 和零，其它视图无供应者时仍保留普通绘制。
pub(crate) trait MeshHitProxyTokenSource {
    fn token_for_instance(&self, stable_instance_key: u64) -> Option<u32>;
}
