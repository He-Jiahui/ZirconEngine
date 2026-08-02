use zircon_runtime_interface::math::UVec2;

use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, ConsoleOutputSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
    SceneEntries,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::PreviewEditorData;

impl PreviewEditorData {
    pub(crate) fn into_snapshot(self) -> EditorDataSnapshot {
        let console_output = ConsoleOutputSnapshot::from(self.status_line.clone());
        let selected = self
            .scene_entries
            .iter()
            .filter(|entry| entry.selected)
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        EditorDataSnapshot {
            scene_entries: SceneEntries::from_entries(
                self.scene_entries
                    .into_iter()
                    .map(super::PreviewSceneEntry::into_snapshot)
                    .collect::<Vec<_>>(),
                selected,
            ),
            inspector: self.inspector.map(super::PreviewInspector::into_snapshot),
            status_line: self.status_line,
            console_output,
            status_task_progress: None,
            hovered_axis: self
                .hovered_axis
                .map(super::PreviewGizmoAxis::into_gizmo_axis),
            viewport_size: UVec2::new(self.viewport_size[0], self.viewport_size[1]),
            scene_viewport_settings: self.scene_viewport_settings,
            mesh_import_path: self.mesh_import_path,
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: self.project_path,
            session_mode: if self.project_open {
                EditorSessionMode::Project
            } else {
                EditorSessionMode::Welcome
            },
            welcome: WelcomePaneSnapshot::default(),
            project_open: self.project_open,
            can_undo: self.can_undo,
            can_redo: self.can_redo,
            bridge_diagnostics: Default::default(),
        }
    }
}
