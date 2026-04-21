mod api;
mod catalog;
mod editor_meta;
mod manager;
mod preview;
mod records;
mod reference_graph;
mod resolver;

pub use api::EditorAssetManager;
pub use catalog::AssetCatalogRecord;
pub(crate) use editor_meta::{editor_meta_path_for_source, EditorAssetMetaDocument};
pub use manager::DefaultEditorAssetManager;
pub use preview::{PreviewArtifactKey, PreviewCache, PreviewScheduler};
pub use records::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChange,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord,
};
pub use reference_graph::ReferenceGraph;
pub use resolver::{resolve_editor_asset_manager, EditorAssetManagerHandle};
