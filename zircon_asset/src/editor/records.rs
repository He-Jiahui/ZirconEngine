use serde::{Deserialize, Serialize};

use crate::{PreviewState, ResourceKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetFolderRecord {
    pub folder_id: String,
    pub parent_folder_id: Option<String>,
    pub locator_prefix: String,
    pub display_name: String,
    pub child_folder_ids: Vec<String>,
    pub direct_asset_uuids: Vec<String>,
    pub recursive_asset_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetCatalogRecord {
    pub uuid: String,
    pub id: String,
    pub locator: String,
    pub kind: ResourceKind,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub preview_state: PreviewState,
    pub meta_path: String,
    pub preview_artifact_path: String,
    pub source_mtime_unix_ms: u64,
    pub source_hash: String,
    pub dirty: bool,
    pub diagnostics: Vec<String>,
    pub direct_reference_uuids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetReferenceRecord {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<ResourceKind>,
    pub known_project_asset: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetDetailsRecord {
    pub asset: EditorAssetCatalogRecord,
    pub direct_references: Vec<EditorAssetReferenceRecord>,
    pub referenced_by: Vec<EditorAssetReferenceRecord>,
    pub editor_adapter: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetCatalogSnapshotRecord {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub library_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub folders: Vec<EditorAssetFolderRecord>,
    pub assets: Vec<EditorAssetCatalogRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetChangeKind {
    CatalogChanged,
    PreviewChanged,
    ReferenceChanged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAssetChangeRecord {
    pub kind: EditorAssetChangeKind,
    pub catalog_revision: u64,
    pub uuid: Option<String>,
    pub locator: Option<String>,
}
