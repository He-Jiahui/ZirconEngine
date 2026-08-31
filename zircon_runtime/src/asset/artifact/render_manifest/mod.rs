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
    RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION, RenderArtifactBlockCodec,
    RenderArtifactBlockDescriptor, RenderArtifactContentId, RenderArtifactLayout,
    RenderArtifactManifest, RenderArtifactMeshBounds, RenderArtifactMeshIndexFormat,
    RenderArtifactMeshLayout, RenderArtifactMeshLodLayout, RenderArtifactMeshLodUploadLayout,
    RenderArtifactMeshVertexFormat, RenderArtifactResidencyClass, RenderArtifactTextureBlockFormat,
    RenderArtifactTextureLayout, RenderArtifactTextureSubresourceLayout, RenderSubresourceId,
};
pub use cook::{
    RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1, RenderArtifactCookOutput, RenderArtifactCookedBlock,
    RenderArtifactMeshCookError, RenderArtifactMeshCookSettings, RenderArtifactTextureCookError,
    RenderArtifactTextureCookSettings, cook_mesh_render_artifact, cook_texture_render_artifact,
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
    RenderArtifactCookPublicationError, RenderArtifactCookPublicationReport,
    RenderArtifactPublishStatus, RenderArtifactStore, RenderArtifactStoreError,
    RenderArtifactStoreLimits, publish_render_artifact_cook_output,
};
pub use validation::RenderArtifactManifestError;

#[cfg(test)]
mod tests;
