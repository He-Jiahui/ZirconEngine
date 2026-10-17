#[path = "cache_key.rs"]
mod cache_key;
#[path = "cache_payload.rs"]
mod cache_payload;
#[path = "chunk_residency.rs"]
mod chunk_residency;
#[path = "ibl_bake_artifact_asset_derived.rs"]
mod ibl_bake_artifact_asset_derived;
#[path = "ibl_bake_artifact_cache.rs"]
mod ibl_bake_artifact_cache;
#[path = "ibl_bake_artifact_runtime_dispatch.rs"]
mod ibl_bake_artifact_runtime_dispatch;
#[path = "ibl_bake_artifact_runtime_writeback.rs"]
mod ibl_bake_artifact_runtime_writeback;
#[path = "ibl_source_cubemap_bundle_manifest/mod.rs"]
mod ibl_source_cubemap_bundle_manifest;
#[path = "ibl_source_cubemap_staging.rs"]
mod ibl_source_cubemap_staging;
#[path = "render_manifest/mod.rs"]
mod render_manifest;
#[path = "store.rs"]
mod store;

pub(crate) use ibl_source_cubemap_bundle_manifest::IblSourceImageIdentity;

pub use cache_key::LibraryCacheKey;
pub use chunk_residency::{
    ArtifactChunkDescriptor, ArtifactChunkInventory, ArtifactChunkResidencyDiagnostics,
};
pub use ibl_bake_artifact_asset_derived::{
    IblBakeArtifactAssetDerivedError, IblBakeArtifactAssetDerivedRead,
    IblBakeArtifactAssetDerivedStore, IblBakeArtifactAssetDerivedWriteReport,
    IBL_BAKE_ASSET_DERIVED_DIRECTORY, IBL_BAKE_ASSET_DERIVED_EXTENSION,
};
pub use ibl_bake_artifact_cache::{
    IblBakeArtifactCacheError, IblBakeArtifactCacheRead, IblBakeArtifactCacheStore,
    IBL_BAKE_RUNTIME_CACHE_DIRECTORY, IBL_BAKE_RUNTIME_CACHE_EXTENSION,
};
pub use ibl_bake_artifact_runtime_dispatch::{
    resolve_ibl_bake_artifact_runtime_dispatch, write_ibl_bake_artifact_runtime_dispatch_readback,
    IblBakeArtifactRuntimeDispatchError, IblBakeArtifactRuntimeDispatchReadbackReport,
    IblBakeArtifactRuntimeDispatchReadbackStatus, IblBakeArtifactRuntimeDispatchReport,
};
pub use ibl_bake_artifact_runtime_writeback::{
    write_ibl_bake_artifact_runtime_readback, IblBakeArtifactRuntimeWritebackError,
    IblBakeArtifactRuntimeWritebackReport, IblBakeArtifactRuntimeWritebackStatus,
};
pub use ibl_source_cubemap_staging::{
    IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingError, IblSourceCubemapStagingRead,
    IblSourceCubemapStagingStore, IblSourceCubemapZcubeWriteReport,
    IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY, IBL_SOURCE_CUBEMAP_STAGING_EXTENSION,
};
pub use render_manifest::{
    cook_mesh_render_artifact, cook_texture_render_artifact, publish_render_artifact_cook_output,
    RenderArtifactBlockAdmissionError, RenderArtifactBlockCancelReason, RenderArtifactBlockCodec,
    RenderArtifactBlockDescriptor, RenderArtifactBlockFailure, RenderArtifactBlockFailureCode,
    RenderArtifactBlockIoDispatchBudget, RenderArtifactBlockIoDispatchError,
    RenderArtifactBlockIoDispatchReport, RenderArtifactBlockLoadStage, RenderArtifactBlockLoader,
    RenderArtifactBlockLoaderCloseReport, RenderArtifactBlockLoaderDiagnostics,
    RenderArtifactBlockLoaderInitError, RenderArtifactBlockLoaderLimits,
    RenderArtifactBlockMaintenanceReport, RenderArtifactBlockPoll, RenderArtifactBlockRequest,
    RenderArtifactBlockTicket, RenderArtifactBlockTicketBatch, RenderArtifactContentId,
    RenderArtifactCookOutput, RenderArtifactCookPublicationError,
    RenderArtifactCookPublicationReport, RenderArtifactCookedBlock, RenderArtifactDecodedBlock,
    RenderArtifactIoPriority, RenderArtifactLayout, RenderArtifactLoadBatch,
    RenderArtifactLoadPlan, RenderArtifactLoadPlanError, RenderArtifactLoadScope,
    RenderArtifactManifest, RenderArtifactManifestAdmissionError,
    RenderArtifactManifestCancelReason, RenderArtifactManifestError, RenderArtifactManifestFailure,
    RenderArtifactManifestFailureCode, RenderArtifactManifestIoDispatchBudget,
    RenderArtifactManifestIoDispatchError, RenderArtifactManifestIoDispatchReport,
    RenderArtifactManifestLoadStage, RenderArtifactManifestLoader,
    RenderArtifactManifestLoaderCloseReport, RenderArtifactManifestLoaderDiagnostics,
    RenderArtifactManifestLoaderInitError, RenderArtifactManifestLoaderLimits,
    RenderArtifactManifestMaintenanceReport, RenderArtifactManifestPoll,
    RenderArtifactManifestRequest, RenderArtifactManifestRequestKey, RenderArtifactManifestTicket,
    RenderArtifactManifestTicketBatch, RenderArtifactMeshBounds, RenderArtifactMeshCookError,
    RenderArtifactMeshCookSettings, RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout,
    RenderArtifactMeshLodLayout, RenderArtifactMeshLodUploadLayout, RenderArtifactMeshVertexFormat,
    RenderArtifactPublishStatus, RenderArtifactResidencyClass, RenderArtifactStore,
    RenderArtifactStoreError, RenderArtifactStoreLimits, RenderArtifactTextureBlockFormat,
    RenderArtifactTextureCookError, RenderArtifactTextureCookSettings, RenderArtifactTextureLayout,
    RenderArtifactTextureSubresourceLayout, RenderSubresourceId,
    RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION, RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1,
};
pub use store::ArtifactStore;

#[path = "raw.rs"]
mod raw;
pub use raw::{
    read_raw_asset, write_raw_asset, RawAssetCatalog, RawAssetCatalogEntry, RawAssetError,
    RawSceneCatalogEntry, RAW_ASSET_MAX_BYTES, RAW_CATALOG_MAX_BYTES,
};

#[cfg(feature = "native-host")]
mod web_bundle;
#[cfg(feature = "native-host")]
pub use web_bundle::{WebBundleError, WebBundleWriter, WEBGPU_ASSET_TARGET};
