mod contract;
mod cook;
mod io_frontier;
mod io_priority;
mod loader;
mod manifest_loader;
mod plan;
mod store;
mod validation;

pub use contract::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactMeshBounds,
    RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout, RenderArtifactMeshLodLayout,
    RenderArtifactMeshLodUploadLayout, RenderArtifactMeshVertexFormat,
    RenderArtifactResidencyClass, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderArtifactTextureSubresourceLayout, RenderSubresourceId,
    RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION,
};
pub use cook::{
    cook_mesh_render_artifact, cook_texture_render_artifact, RenderArtifactCookOutput,
    RenderArtifactCookedBlock, RenderArtifactMeshCookError, RenderArtifactMeshCookSettings,
    RenderArtifactTextureCookError, RenderArtifactTextureCookSettings,
    RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1,
};
pub use io_priority::RenderArtifactIoPriority;
pub use loader::{
    RenderArtifactBlockAdmissionError, RenderArtifactBlockCancelReason, RenderArtifactBlockFailure,
    RenderArtifactBlockFailureCode, RenderArtifactBlockIoDispatchBudget,
    RenderArtifactBlockIoDispatchError, RenderArtifactBlockIoDispatchReport,
    RenderArtifactBlockLoadStage, RenderArtifactBlockLoader, RenderArtifactBlockLoaderCloseReport,
    RenderArtifactBlockLoaderDiagnostics, RenderArtifactBlockLoaderInitError,
    RenderArtifactBlockLoaderLimits, RenderArtifactBlockMaintenanceReport, RenderArtifactBlockPoll,
    RenderArtifactBlockRequest, RenderArtifactBlockTicket, RenderArtifactBlockTicketBatch,
    RenderArtifactDecodedBlock,
};
pub use manifest_loader::{
    RenderArtifactManifestAdmissionError, RenderArtifactManifestCancelReason,
    RenderArtifactManifestFailure, RenderArtifactManifestFailureCode,
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestIoDispatchError,
    RenderArtifactManifestIoDispatchReport, RenderArtifactManifestLoadStage,
    RenderArtifactManifestLoader, RenderArtifactManifestLoaderCloseReport,
    RenderArtifactManifestLoaderDiagnostics, RenderArtifactManifestLoaderInitError,
    RenderArtifactManifestLoaderLimits, RenderArtifactManifestMaintenanceReport,
    RenderArtifactManifestPoll, RenderArtifactManifestRequest, RenderArtifactManifestRequestKey,
    RenderArtifactManifestTicket, RenderArtifactManifestTicketBatch,
};
pub use plan::{
    RenderArtifactLoadBatch, RenderArtifactLoadPlan, RenderArtifactLoadPlanError,
    RenderArtifactLoadScope,
};
pub use store::{
    publish_render_artifact_cook_output, RenderArtifactCookPublicationError,
    RenderArtifactCookPublicationReport, RenderArtifactPublishStatus, RenderArtifactStore,
    RenderArtifactStoreError, RenderArtifactStoreLimits,
};
pub use validation::RenderArtifactManifestError;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
