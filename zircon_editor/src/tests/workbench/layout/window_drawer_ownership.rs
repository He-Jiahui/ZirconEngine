use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, LayoutCommand, LayoutManager,
    WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
fn default_workbench_layout_seeds_drawers_inside_workbench_activity_window() {
    let layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");
    let activity_windows = layout.activity_windows();
    let workbench_window = activity_windows
        .get(&window_id)
        .expect("default layout should include workbench activity window");

    assert_eq!(workbench_window.descriptor_id.0, "editor.workbench_window");
    assert_eq!(
        workbench_window.activity_drawers.len(),
        ActivityDrawerSlot::ALL.len()
    );
    assert!(workbench_window
        .activity_drawers
        .contains_key(&ActivityDrawerSlot::LeftTop));
    assert!(workbench_window
        .activity_drawers
        .contains_key(&ActivityDrawerSlot::BottomRight));
}

#[test]
fn default_workbench_layout_stores_activity_window_drawers_as_layout_state() {
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");

    layout.drawers.clear();

    let workbench_window = layout
        .activity_windows
        .get(&window_id)
        .expect("activity window drawers should be stored on layout state");
    assert_eq!(
        workbench_window.activity_drawers.len(),
        ActivityDrawerSlot::ALL.len()
    );
    assert!(workbench_window
        .activity_drawers
        .contains_key(&ActivityDrawerSlot::LeftTop));
}

#[test]
fn drawer_layout_commands_mutate_default_activity_window_drawers() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: 144.0,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerMode {
                slot: ActivityDrawerSlot::LeftTop,
                mode: ActivityDrawerMode::Collapsed,
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(drawer.extent, 144.0);
    assert_eq!(drawer.mode, ActivityDrawerMode::Collapsed);
}

#[test]
fn drawer_attach_focus_and_close_commands_mutate_default_activity_window_drawers() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");
    let hierarchy = ViewInstanceId::new("editor.hierarchy#1");
    let inspector = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: hierarchy.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: inspector.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::FocusView {
                instance_id: hierarchy.clone(),
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(
        drawer.tab_stack.tabs,
        vec![hierarchy.clone(), inspector.clone()]
    );
    assert_eq!(drawer.tab_stack.active_tab, Some(hierarchy.clone()));
    assert_eq!(drawer.active_view, Some(hierarchy.clone()));

    manager
        .apply(
            &mut layout,
            LayoutCommand::CloseView {
                instance_id: hierarchy.clone(),
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(drawer.tab_stack.tabs, vec![inspector.clone()]);
    assert_eq!(drawer.active_view, Some(inspector));
}
