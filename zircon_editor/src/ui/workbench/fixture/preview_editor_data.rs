use crate::scene::viewport::SceneViewportChromeSettings;
use serde::{Deserialize, Serialize};

use super::{PreviewGizmoAxis, PreviewInspector, PreviewSceneEntry};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// 供可重现预览与测试复用的编辑数据；字段只描述样本输入，不直接驱动真实项目会话。
pub struct PreviewEditorData {
    pub scene_entries: Vec<PreviewSceneEntry>,
    pub inspector: Option<PreviewInspector>,
    pub status_line: String,
    pub hovered_axis: Option<PreviewGizmoAxis>,
    pub viewport_size: [u32; 2],
    #[serde(default)]
    pub scene_viewport_settings: SceneViewportChromeSettings,
    pub mesh_import_path: String,
    pub project_path: String,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}
