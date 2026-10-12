#[path = "contract.rs"]
mod contract;
#[path = "plan.rs"]
mod plan;
#[path = "submit.rs"]
mod submit;

pub(crate) use contract::{
    RenderAssetGpuUploadBudgetClass, RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlanError,
    RenderAssetGpuUploadQuote,
};
pub(crate) use plan::{RenderAssetGpuUploadPlan, RenderAssetGpuUploadPlanKind};
pub(super) use submit::RenderAssetGpuUploadFinalize;
pub(crate) use submit::{
    RenderAssetGpuArtifact, RenderAssetGpuArtifactKind, RenderAssetGpuMeshArtifact,
    RenderAssetGpuMeshLod, RenderAssetGpuTextureArtifact, RenderAssetGpuUploadLease,
    RenderAssetGpuUploadSubmitError,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
