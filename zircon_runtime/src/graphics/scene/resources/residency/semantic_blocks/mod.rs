#[path = "../../../residency/semantic_blocks/contract.rs"]
mod contract;
#[path = "load.rs"]
mod load;
#[path = "../../../residency/semantic_blocks/semantic_load.rs"]
mod semantic_load;

pub(crate) use contract::{
    RenderAssetCpuBlockLease, RenderAssetSemanticBlockLoadAdvance,
    RenderAssetSemanticBlockLoadError,
};
pub(crate) use load::RenderAssetSemanticBlockLoad;
pub(crate) use semantic_load::{
    RenderAssetCpuArtifactLease, RenderAssetSemanticLoad, RenderAssetSemanticLoadAdvance,
    RenderAssetSemanticLoadError, RenderAssetSemanticLoadStage,
};
