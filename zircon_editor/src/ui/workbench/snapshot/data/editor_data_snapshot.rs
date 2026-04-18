use crate::GizmoAxis;
use zircon_math::UVec2;
use zircon_scene::SceneViewportSettings;

use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::super::asset::AssetWorkspaceSnapshot;
use super::{InspectorSnapshot, ProjectOverviewSnapshot, SceneEntry};

#[derive(Clone, Debug)]
pub struct EditorDataSnapshot {
    pub scene_entries: Vec<SceneEntry>,
    pub inspector: Option<InspectorSnapshot>,
    pub status_line: String,
    pub hovered_axis: Option<GizmoAxis>,
    pub viewport_size: UVec2,
    pub scene_viewport_settings: SceneViewportSettings,
    pub mesh_import_path: String,
    pub project_overview: ProjectOverviewSnapshot,
    pub asset_activity: AssetWorkspaceSnapshot,
    pub asset_browser: AssetWorkspaceSnapshot,
    pub project_path: String,
    pub session_mode: EditorSessionMode,
    pub welcome: WelcomePaneSnapshot,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}
