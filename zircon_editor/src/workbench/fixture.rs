//! Shared preview fixtures used by browser and desktop workbench hosts.

use serde::{Deserialize, Serialize};

use crate::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, InspectorSnapshot,
    ProjectOverviewSnapshot, SceneEntry,
};
use crate::{ViewDescriptor, ViewInstance, WorkbenchLayout};
use zircon_graphics::GizmoAxis;
use zircon_math::UVec2;
use crate::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

const DEFAULT_LAYOUT_JSON: &str = include_str!("../../fixtures/workbench/default-layout.json");
const DEFAULT_DESCRIPTORS_JSON: &str =
    include_str!("../../fixtures/workbench/view-descriptors.json");
const DEFAULT_INSTANCES_JSON: &str = include_str!("../../fixtures/workbench/view-instances.json");
const DEFAULT_EDITOR_DATA_JSON: &str = include_str!("../../fixtures/workbench/editor-data.json");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewEditorData {
    pub scene_entries: Vec<PreviewSceneEntry>,
    pub inspector: Option<PreviewInspector>,
    pub status_line: String,
    pub hovered_axis: Option<PreviewGizmoAxis>,
    pub viewport_size: [u32; 2],
    pub mesh_import_path: String,
    pub project_path: String,
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

impl PreviewEditorData {
    fn into_snapshot(self) -> EditorDataSnapshot {
        EditorDataSnapshot {
            scene_entries: self
                .scene_entries
                .into_iter()
                .map(PreviewSceneEntry::into_snapshot)
                .collect(),
            inspector: self.inspector.map(PreviewInspector::into_snapshot),
            status_line: self.status_line,
            hovered_axis: self.hovered_axis.map(PreviewGizmoAxis::into_gizmo_axis),
            viewport_size: UVec2::new(self.viewport_size[0], self.viewport_size[1]),
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewSceneEntry {
    pub id: u64,
    pub name: String,
    pub depth: usize,
    pub selected: bool,
}

impl PreviewSceneEntry {
    fn into_snapshot(self) -> SceneEntry {
        SceneEntry {
            id: self.id,
            name: self.name,
            depth: self.depth,
            selected: self.selected,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewInspector {
    pub id: u64,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
}

impl PreviewInspector {
    fn into_snapshot(self) -> InspectorSnapshot {
        InspectorSnapshot {
            id: self.id,
            name: self.name,
            parent: self.parent,
            translation: self.translation,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PreviewGizmoAxis {
    X,
    Y,
    Z,
}

impl PreviewGizmoAxis {
    fn into_gizmo_axis(self) -> GizmoAxis {
        match self {
            Self::X => GizmoAxis::X,
            Self::Y => GizmoAxis::Y,
            Self::Z => GizmoAxis::Z,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewFixture {
    pub layout: WorkbenchLayout,
    pub descriptors: Vec<ViewDescriptor>,
    pub instances: Vec<ViewInstance>,
    pub editor: PreviewEditorData,
}

impl PreviewFixture {
    pub fn build_chrome(&self) -> EditorChromeSnapshot {
        EditorChromeSnapshot::build(
            self.editor.clone().into_snapshot(),
            &self.layout,
            self.instances.clone(),
            self.descriptors.clone(),
        )
    }
}

pub fn default_preview_fixture() -> PreviewFixture {
    PreviewFixture {
        layout: serde_json::from_str(DEFAULT_LAYOUT_JSON).expect("preview layout fixture"),
        descriptors: serde_json::from_str(DEFAULT_DESCRIPTORS_JSON)
            .expect("preview view descriptors fixture"),
        instances: serde_json::from_str(DEFAULT_INSTANCES_JSON)
            .expect("preview view instances fixture"),
        editor: serde_json::from_str(DEFAULT_EDITOR_DATA_JSON)
            .expect("preview editor data fixture"),
    }
}
