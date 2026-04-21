use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportSettings, SceneViewportTool,
    TransformSpace, ViewOrientation,
};
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

#[test]
fn chrome_builder_carries_scene_viewport_settings_into_snapshot() {
    let settings = SceneViewportSettings {
        tool: SceneViewportTool::Scale,
        transform_space: TransformSpace::Global,
        projection_mode: ProjectionMode::Orthographic,
        view_orientation: ViewOrientation::PosY,
        gizmos_enabled: false,
        display_mode: DisplayMode::WireOverlay,
        grid_mode: GridMode::VisibleAndSnap,
        translate_step: 2.5,
        rotate_step_deg: 30.0,
        scale_step: 0.25,
        preview_lighting: false,
        preview_skybox: false,
    };
    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Ready".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: settings.clone(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Project,
            welcome: WelcomePaneSnapshot::default(),
            project_open: true,
            can_undo: false,
            can_redo: false,
        },
        &WorkbenchLayout::default(),
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(chrome.scene_viewport_settings, settings);
}
