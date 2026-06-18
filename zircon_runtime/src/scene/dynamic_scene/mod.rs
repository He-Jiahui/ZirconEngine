//! Runtime dynamic scene snapshots backed by reflected components and resources.

mod document;
mod entity;
mod error;
mod patch;
mod remap;
mod scene;
mod session;
mod value;

pub use entity::{DynamicComponent, DynamicEntity, DynamicResource};
pub use error::DynamicSceneError;
pub use patch::ScenePatch;
pub use remap::EntityRemap;
pub use scene::{DynamicScene, DYNAMIC_SCENE_FORMAT_VERSION};
pub use session::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
    RuntimeSessionArchivePathStatus, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveStatistics,
    RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata, RuntimeSessionSlot,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotSummary,
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};
