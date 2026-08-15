//! Runtime dynamic scene snapshots backed by reflected components and resources.

mod asset_reload;
mod document;
mod entity;
mod error;
mod patch;
mod remap;
mod scene;
mod scene_asset;
mod session;
mod spawn_task;
mod value;

pub use asset_reload::{
    DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure,
    DynamicSceneAssetReloadApplyReport, DynamicSceneAssetReloadDiagnostics,
    DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadFrameApplyReport,
    DynamicSceneAssetReloadLimits, DynamicSceneAssetReloadPendingReport,
    DynamicSceneAssetReloadPendingTaskSnapshot, DynamicSceneAssetReloadQueue,
    DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent,
    DynamicSceneAssetReloadStaleResult, DynamicSceneAssetReloadSupersededTask,
    DynamicSceneAssetReloadTask,
};
pub use entity::{DynamicComponent, DynamicEntity, DynamicResource};
pub use error::DynamicSceneError;
pub use patch::{
    ScenePatch, ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap,
    ScenePatchPreviewReport, ScenePatchPreviewResource,
};
pub use remap::EntityRemap;
pub use scene::DynamicScene;
pub(crate) use scene::{CompiledSceneSpawn, PreflightedSceneMutation};
pub use session::{
    RuntimeSessionArchive, RuntimeSessionArchiveArtifact, RuntimeSessionArchiveArtifactDiagnostics,
    RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, RuntimeSessionArchivePathStatus,
    RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveStatistics,
    RuntimeSessionArchiveWriteSubmission, RuntimeSessionArchiveWriter,
    RuntimeSessionArchiveWriterLimits, RuntimeSessionArchiveWriterSubmitError,
    RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelectionReport,
    RuntimeSessionSlotSelector, RuntimeSessionSlotSummary,
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};
pub(crate) use spawn_task::{DynamicSceneSpawnTargetSnapshot, StagedDynamicSceneSpawn};
pub use spawn_task::{DynamicSceneSpawnTask, PreparedDynamicSceneSpawn};
