use zircon_runtime_interface::math::UVec2;

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportChromeSettings, TransformHandleKind,
    TransformSpace, ViewOrientation,
};
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

#[test]
fn chrome_builder_carries_scene_viewport_settings_into_snapshot() {
    let settings = SceneViewportChromeSettings {
        mode: SceneModeActivation::Transform(TransformHandleKind::Scale),
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
            scene_entries: Default::default(),
            inspector: None,
            status_line: "Ready".to_string(),
            console_output: "Ready".into(),
            status_task_progress: None,
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
            bridge_diagnostics: Default::default(),
        },
        &WorkbenchLayout::default(),
        Vec::new(),
        Vec::new(),
        None,
    );

    assert_eq!(chrome.scene_viewport_settings, settings);
}
