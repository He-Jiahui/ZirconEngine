//! 当前活动窗口的菜单偏好及默认窗口模板只读投影。
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::SceneViewportChromeSettings;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, MainPageSnapshot,
    ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewKind,
};
use crate::ui::workbench::window_registry::MenuOverflowMode;

#[test]
/// 验证偏好来自活动窗口而非全局常量，供菜单呈现选择溢出方式。
fn chrome_builder_reads_active_window_menu_overflow_preference() {
    let mut layout = WorkbenchLayout::default();
    layout
        .default_activity_window_mut()
        .expect("default workbench window")
        .menu_overflow_mode = MenuOverflowMode::MultiColumn;

    let chrome =
        EditorChromeSnapshot::build(empty_editor_data(), &layout, Vec::new(), Vec::new(), None);

    assert_eq!(chrome.menu_overflow_mode, MenuOverflowMode::MultiColumn);
}

#[test]
/// 核对默认工作窗口描述符模板能跨chrome边界保留，实际模板加载由宿主另行完成。
fn chrome_builder_carries_default_workbench_window_template() {
    let layout = WorkbenchLayout::default();
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.workbench_window"),
        ViewKind::ActivityWindow,
        "Workbench",
    )
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/windows/workbench_window.zui",
    ))];

    let chrome =
        EditorChromeSnapshot::build(empty_editor_data(), &layout, Vec::new(), descriptors, None);

    let MainPageSnapshot::Workbench {
        activity_window_template,
        ..
    } = &chrome.workbench.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    assert_eq!(
        activity_window_template
            .as_ref()
            .map(|template| template.document_id.as_str()),
        Some("res://ui/editor/windows/workbench_window.zui")
    );
}

/// 给菜单和模板投影提供无项目数据，避免场景状态影响壳层断言。
fn empty_editor_data() -> EditorDataSnapshot {
    EditorDataSnapshot {
        scene_entries: Default::default(),
        inspector: None,
        status_line: "Ready".to_string(),
        console_output: "Ready".into(),
        status_task_progress: None,
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportChromeSettings::default(),
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
        bridge_diagnostics: Default::default(),
    }
}
