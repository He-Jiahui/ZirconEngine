use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::window_registry::MenuOverflowMode;

#[test]
fn chrome_builder_reads_active_window_menu_overflow_preference() {
    let mut layout = WorkbenchLayout::default();
    layout
        .default_activity_window_mut()
        .expect("default workbench window")
        .menu_overflow_mode = MenuOverflowMode::MultiColumn;

    let chrome = EditorChromeSnapshot::build(empty_editor_data(), &layout, Vec::new(), Vec::new());

    assert_eq!(chrome.menu_overflow_mode, MenuOverflowMode::MultiColumn);
}

fn empty_editor_data() -> EditorDataSnapshot {
    EditorDataSnapshot {
        scene_entries: Vec::new(),
        inspector: None,
        status_line: "Ready".to_string(),
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportSettings::default(),
        mesh_import_path: String::new(),
        project_overview: ProjectOverviewSnapshot::default(),
        asset_activity: AssetWorkspaceSnapshot::default(),
        asset_browser: AssetWorkspaceSnapshot::default(),
        project_path: String::new(),
        session_mode: EditorSessionMode::Welcome,
        welcome: WelcomePaneSnapshot::default(),
        project_open: false,
        can_undo: false,
        can_redo: false,
    }
}
