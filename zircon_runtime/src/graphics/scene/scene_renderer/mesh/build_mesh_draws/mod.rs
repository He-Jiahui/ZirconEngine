mod build;
mod create_mesh_draw;
mod indexed_indirect_args;
mod raster_draws_for_mesh;

pub(crate) use build::{
    BuiltMeshDraws, MaterialPipelineFeatureSet, MaterialPipelineRequirementCensus,
    PendingMeshCommandCacheExtractionContext, PendingMeshCommandCacheExtractionStats,
    PendingMeshCommandCachePlanStats, build_mesh_draws,
};
pub(crate) use indexed_indirect_args::IndexedIndirectArgs;

pub(crate) trait MeshHitProxyTokenSource {
    fn token_for_instance(&self, stable_instance_key: u64) -> Option<u32>;
}
