//! Runtime dynamic scene snapshots backed by reflected components and resources.

#[cfg(feature = "native-host")]
#[path = "asset_reload/mod.rs"]
mod asset_reload;
#[path = "document/mod.rs"]
mod document;
#[path = "entity/mod.rs"]
mod entity;
#[path = "error.rs"]
mod error;
#[path = "patch.rs"]
mod patch;
#[path = "remap.rs"]
mod remap;
#[path = "scene/mod.rs"]
mod scene;
#[cfg(feature = "native-host")]
#[path = "scene_asset/mod.rs"]
mod scene_asset;
#[cfg(feature = "native-host")]
#[path = "session/mod.rs"]
mod session;
#[cfg(feature = "native-host")]
#[path = "spawn_task/mod.rs"]
mod spawn_task;
#[path = "value/mod.rs"]
mod value;

#[cfg(feature = "native-host")]
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
#[cfg(feature = "native-host")]
pub use session::{
    RuntimeSessionArchive, RuntimeSessionArchiveArtifact, RuntimeSessionArchiveArtifactDiagnostics,
    RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, RuntimeSessionArchivePathStatus,
    RuntimeSessionArchivePruneReport, RuntimeSessionArchiveReadArtifact,
    RuntimeSessionArchiveReadOutcome, RuntimeSessionArchiveReadSubmission,
    RuntimeSessionArchiveReader, RuntimeSessionArchiveReaderDiagnostics,
    RuntimeSessionArchiveReaderLimits, RuntimeSessionArchiveReaderSubmitError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveSavePreviewReport,
    RuntimeSessionArchiveStatistics, RuntimeSessionArchiveWriteSubmission,
    RuntimeSessionArchiveWriter, RuntimeSessionArchiveWriterLimits,
    RuntimeSessionArchiveWriterSubmitError, RuntimeSessionLevelRestoreReport,
    RuntimeSessionMetadata, RuntimeSessionSlot, RuntimeSessionSlotCapturePreviewReport,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotExportPreviewReport,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelectionReport, RuntimeSessionSlotSelector, RuntimeSessionSlotSummary,
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};
#[cfg(feature = "native-host")]
pub(crate) use spawn_task::{DynamicSceneSpawnTargetSnapshot, StagedDynamicSceneSpawn};
#[cfg(feature = "native-host")]
pub use spawn_task::{DynamicSceneSpawnTask, PreparedDynamicSceneSpawn};
