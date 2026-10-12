#[path = "api.rs"]
mod api;
#[path = "catalog.rs"]
mod catalog;
#[path = "change_stream.rs"]
mod change_stream;
#[path = "error.rs"]
mod error;
#[path = "generation.rs"]
mod generation;
#[path = "handle.rs"]
mod handle;
#[path = "manager/mod.rs"]
mod manager;
#[path = "preview.rs"]
mod preview;
#[path = "records.rs"]
mod records;

pub use api::EditorAssetManager;
pub use catalog::AssetCatalogRecord;
pub(crate) use change_stream::EditorAssetChangeHub;
pub use change_stream::{EditorAssetChangeDelivery, EditorAssetChangeSubscription};
pub use error::EditorAssetSyncError;
pub use generation::{EditorAssetCatalogGeneration, EditorAssetDetailsGeneration};
pub use handle::editor_asset_manager_handle;
pub use manager::DefaultEditorAssetManager;
pub(crate) use preview::PreviewJobToken;
pub use preview::{PreviewArtifactKey, PreviewCache, PreviewScheduler};
pub use records::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChange,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord, EditorAssetSubassetRecord,
};
