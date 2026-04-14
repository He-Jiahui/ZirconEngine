use serde::{Deserialize, Serialize};

use crate::WorldHandle;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSummary {
    pub handle: WorldHandle,
    pub entity_count: usize,
    pub selected_entity: Option<u64>,
    pub active_camera: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetPipelineInfo {
    pub default_worker_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root_path: String,
    pub name: String,
    pub default_scene_uri: String,
    pub library_version: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRecordKind {
    Texture,
    Shader,
    Material,
    Scene,
    Model,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetStatusRecord {
    pub id: String,
    pub uri: String,
    pub kind: AssetRecordKind,
    pub artifact_uri: Option<String>,
    pub imported: bool,
    pub source_hash: String,
    pub importer_version: u32,
    pub config_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetChangeKind {
    Added,
    Modified,
    Removed,
    Renamed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetChangeRecord {
    pub kind: AssetChangeKind,
    pub uri: String,
    pub previous_uri: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewStateRecord {
    Dirty,
    Ready,
    Error,
}

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
    pub kind: AssetRecordKind,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub preview_state: PreviewStateRecord,
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
    pub kind: Option<AssetRecordKind>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStateRecord {
    Pending,
    Ready,
    Error,
    Reloading,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceStatusRecord {
    pub id: String,
    pub locator: String,
    pub kind: AssetRecordKind,
    pub artifact_locator: Option<String>,
    pub revision: u64,
    pub state: ResourceStateRecord,
    pub dependency_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceChangeKind {
    Added,
    Updated,
    Removed,
    Renamed,
    ReloadFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceChangeRecord {
    pub kind: ResourceChangeKind,
    pub id: String,
    pub locator: Option<String>,
    pub previous_locator: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InputButton {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    Key(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    CursorMoved { x: f32, y: f32 },
    ButtonPressed(InputButton),
    ButtonReleased(InputButton),
    WheelScrolled { delta: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEventRecord {
    pub sequence: u64,
    pub timestamp_millis: u64,
    pub event: InputEvent,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub cursor_position: [f32; 2],
    pub pressed_buttons: Vec<InputButton>,
    pub wheel_accumulator: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: Vec<String>,
}

impl CapabilitySet {
    pub fn with(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self.capabilities.sort();
        self.capabilities.dedup();
        self
    }
}
