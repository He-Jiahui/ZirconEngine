//! 布局命令结果的serde保留与恢复来源优先级测试；恢复优先级不等于完整结构校验。
use crate::ui::workbench::layout::{
    ActivityDrawerSlot, LayoutCommand, LayoutManager, MainPageId, RestorePolicy, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
/// 应用文档、抽屉和浮窗迁移后检查保存材料；不创建或关闭系统浮窗。
fn layout_manager_moves_views_and_roundtrips_layouts() {
    let mut layout = WorkbenchLayout::default();
    let scene_view = ViewInstanceId::new("scene#1");
    let inspector_view = ViewInstanceId::new("inspector#1");
    let floating_window = MainPageId::new("window#1");
    let manager = LayoutManager::default();

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: scene_view.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![0]),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: inspector_view.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::DetachViewToWindow {
                instance_id: scene_view.clone(),
                new_window: floating_window.clone(),
            },
        )
        .unwrap();

    let json = serde_json::to_string(&layout).unwrap();
    let restored: WorkbenchLayout = serde_json::from_str(&json).unwrap();

    assert!(restored
        .active_activity_window_drawers()
        .get(&ActivityDrawerSlot::RightTop)
        .unwrap()
        .tab_stack
        .tabs
        .contains(&inspector_view));
    assert!(restored
        .floating_windows
        .iter()
        .any(|window| window.window_id == floating_window));
}

#[test]
/// 用不同活动页标记区分来源，验证项目优先回退策略；标记页刻意只用来识别选中的载荷。
fn restore_policy_prefers_project_workspace_before_global_default() {
    let mut global = WorkbenchLayout::default();
    global.active_main_page = MainPageId::new("global");
    let mut project = WorkbenchLayout::default();
    project.active_main_page = MainPageId::new("project");
    let manager = LayoutManager::default();

    let restored = manager
        .restore_workspace(
            RestorePolicy::ProjectThenGlobal,
            Some(ProjectEditorWorkspace {
                workbench: project.clone(),
                open_view_instances: Vec::new(),
                focused_view: None,
                active_drawers: Vec::new(),
                scene_viewport_sessions: Default::default(),
            }),
            Some(global),
        )
        .unwrap();

    assert_eq!(restored.active_main_page, project.active_main_page);
}
