use super::*;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn native_window_hosts_allocate_independent_surfaces_per_floating_window() {
    let mut manager = WindowHostManager::default();
    let first = MainPageId::new("window:first");
    let second = MainPageId::new("window:second");

    manager.open_native_window(first.clone(), Some(11));
    manager.open_native_window(second.clone(), Some(22));

    let states = manager.states();
    let first_state = states
        .iter()
        .find(|state| state.window_id == first)
        .expect("first native window state");
    let second_state = states
        .iter()
        .find(|state| state.window_id == second)
        .expect("second native window state");

    assert_eq!(
        first_state.surface_tree_id.0,
        "zircon.editor.native_window.window:first"
    );
    assert_eq!(
        second_state.surface_tree_id.0,
        "zircon.editor.native_window.window:second"
    );
    assert_ne!(first_state.surface_tree_id, second_state.surface_tree_id);
    assert_ne!(
        manager.windows[&first].surface.tree.tree_id,
        manager.windows[&second].surface.tree.tree_id,
        "each floating window must own its own runtime UiSurface instead of sharing the main host surface"
    );
}

#[test]
fn native_window_host_sync_preserves_surface_when_bounds_change() {
    let mut manager = WindowHostManager::default();
    let window_id = MainPageId::new("window:preview");

    manager.sync_layout_windows(&layout_with_floating_window(
        window_id.clone(),
        ShellFrame::new(20.0, 30.0, 320.0, 240.0),
    ));
    let surface_tree_id = manager.windows[&window_id].surface.tree.tree_id.clone();

    manager.sync_layout_windows(&layout_with_floating_window(
        window_id.clone(),
        ShellFrame::new(80.0, 90.0, 640.0, 360.0),
    ));

    assert_eq!(
        manager.windows[&window_id].surface.tree.tree_id,
        surface_tree_id
    );
    assert_eq!(
        manager.states()[0].bounds,
        [80.0, 90.0, 640.0, 360.0],
        "geometry changes should update host bounds without replacing the window-owned surface"
    );
}

#[test]
fn native_window_host_sync_removes_surface_for_stale_floating_window() {
    let mut manager = WindowHostManager::default();
    let window_id = MainPageId::new("window:preview");

    manager.sync_layout_windows(&layout_with_floating_window(
        window_id,
        ShellFrame::new(20.0, 30.0, 320.0, 240.0),
    ));
    assert_eq!(manager.windows.len(), 1);

    manager.sync_layout_windows(&WorkbenchLayout::default());

    assert!(manager.windows.is_empty());
    assert!(manager.states().is_empty());
}

fn layout_with_floating_window(window_id: MainPageId, frame: ShellFrame) -> WorkbenchLayout {
    let view_id = ViewInstanceId::new(format!("{}#view", window_id.0));
    WorkbenchLayout {
        floating_windows: vec![FloatingWindowLayout {
            window_id,
            title: "Preview".to_string(),
            workspace: DocumentNode::tabs(TabStackLayout {
                tabs: vec![view_id.clone()],
                active_tab: Some(view_id.clone()),
            }),
            focused_view: Some(view_id),
            frame,
        }],
        ..WorkbenchLayout::default()
    }
}
