mod api;
mod catalog;
mod change_stream;
mod generation;
mod handle;
mod manager;
mod preview;
mod records;
mod reference_graph;

pub use api::EditorAssetManager;
pub use catalog::AssetCatalogRecord;
pub(crate) use change_stream::EditorAssetChangeHub;
pub use change_stream::{EditorAssetChangeDelivery, EditorAssetChangeSubscription};
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
pub use reference_graph::ReferenceGraph;
