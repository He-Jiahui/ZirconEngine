use crate::core::editor_event::MenuAction;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId, WorkspaceTarget};
use crate::ui::workbench::model::{MainHostStripModel, WorkbenchViewModel};

use super::support::{
    sample_exclusive_chrome, sample_floating_window_chrome, sample_workbench_chrome,
};

#[test]
fn workbench_view_model_projects_menu_strip_drawers_and_status() {
    let chrome = sample_workbench_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    assert_eq!(
        model
            .menu_bar
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect::<Vec<_>>(),
        vec!["File", "Edit", "Selection", "View", "Window", "Help"]
    );
    assert!(model
        .menu_bar
        .menus
        .iter()
        .flat_map(|menu| menu.items.iter())
        .any(|item| item.action.as_ref() == Some(&MenuAction::Undo) && item.enabled));
    let undo_operation = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Edit")
        .and_then(|menu| menu.items.iter().find(|item| item.label == "Undo"))
        .and_then(|item| item.operation_path.as_ref())
        .map(|path| path.as_str())
        .expect("undo operation path");
    assert_eq!(undo_operation, "Edit.History.Undo");
    assert_eq!(model.host_strip.active_page, MainPageId::workbench());
    assert_eq!(
        model
            .host_strip
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Workbench", "Scene"]
    );
    assert!(model.drawer_ring.visible);
    assert!(model
        .drawer_ring
        .drawers
        .contains_key(&ActivityDrawerSlot::LeftTop));
    assert_eq!(model.status_bar.primary_text, "Editor booted");
    assert_eq!(model.status_bar.viewport_label, "1280 x 720");
    let save_project_binding = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "File")
        .and_then(|menu| menu.items.iter().find(|item| item.label == "Save Project"))
        .map(|item| item.binding.native_binding())
        .expect("save project binding");
    assert_eq!(
        save_project_binding,
        r#"WorkbenchMenuBar/SaveProject:onClick(MenuAction("SaveProject"))"#
    );
    let reset_layout_operation = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Window")
        .and_then(|menu| menu.items.iter().find(|item| item.label == "Reset Layout"))
        .and_then(|item| item.operation_path.as_ref())
        .map(|path| path.as_str())
        .expect("reset layout operation path");
    assert_eq!(reset_layout_operation, "Window.Layout.Reset");
}

#[test]
fn workbench_view_model_freezes_drawers_for_exclusive_page() {
    let chrome = sample_exclusive_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    assert!(!model.drawer_ring.visible);
    assert!(matches!(
        model.host_strip.mode,
        MainHostStripModel::ExclusiveWindow { .. }
    ));
    assert_eq!(
        model
            .host_strip
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Prefab Editor", "crate://player.prefab"]
    );
}

#[test]
fn workbench_view_model_exposes_floating_windows_as_workspace_tabs() {
    let chrome = sample_floating_window_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    assert_eq!(model.document_tabs.len(), 1);
    assert_eq!(model.floating_windows.len(), 1);

    let floating = &model.floating_windows[0];
    assert_eq!(floating.window_id, MainPageId::new("window:prefab"));
    assert_eq!(floating.title, "Prefab Popout");
    assert_eq!(
        floating.requested_frame,
        ShellFrame::new(111.0, 92.0, 640.0, 420.0)
    );
    assert_eq!(
        floating.focused_view.as_ref().map(|id| id.0.as_str()),
        Some("editor.prefab#float")
    );
    assert_eq!(
        floating
            .tabs
            .iter()
            .map(|tab| (tab.title.as_str(), tab.workspace_path.clone(), tab.active))
            .collect::<Vec<_>>(),
        vec![("Scene", vec![0], true), ("Prefab Editor", vec![1], true),]
    );
    assert!(floating.tabs.iter().all(|tab| matches!(
        tab.workspace,
        WorkspaceTarget::FloatingWindow(ref window_id) if window_id == &MainPageId::new("window:prefab")
    )));
}
