mod api;
mod catalog;
mod manager;
mod preview;
mod records;
mod reference_graph;
mod resolver;

pub use api::EditorAssetManager;
pub use catalog::AssetCatalogRecord;
pub use manager::DefaultEditorAssetManager;
pub use preview::{PreviewArtifactKey, PreviewCache, PreviewScheduler};
pub use records::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChangeKind,
    EditorAssetChangeRecord, EditorAssetDetailsRecord, EditorAssetFolderRecord,
    EditorAssetReferenceRecord,
};
pub use reference_graph::ReferenceGraph;
pub use resolver::{resolve_editor_asset_manager, EditorAssetManagerHandle};
