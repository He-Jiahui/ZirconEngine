use serde::{Deserialize, Serialize};
use zircon_framework::render::SceneViewportSettings;

use super::{PreviewGizmoAxis, PreviewInspector, PreviewSceneEntry};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewEditorData {
    pub scene_entries: Vec<PreviewSceneEntry>,
    pub inspector: Option<PreviewInspector>,
    pub status_line: String,
    pub hovered_axis: Option<PreviewGizmoAxis>,
    pub viewport_size: [u32; 2],
    #[serde(default)]
    pub scene_viewport_settings: SceneViewportSettings,
    pub mesh_import_path: String,
    pub project_path: String,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}
