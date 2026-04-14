use crate::{
    ActivityDrawerSlot, LayoutCommand, LayoutManager, MainHostPageLayout, MainPageId,
    ProjectEditorWorkspace, RestorePolicy, SplitAxis, SplitPlacement, TabInsertionAnchor,
    TabInsertionSide, ViewHost, ViewInstanceId, WorkbenchLayout, WorkspaceTarget,
};

#[test]
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
        .drawers
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
                layout_version: 1,
                workbench: project.clone(),
                open_view_instances: Vec::new(),
                active_center_tab: None,
                active_drawers: Vec::new(),
            }),
            Some(global),
        )
        .unwrap();

    assert_eq!(restored.active_main_page, project.active_main_page);
}

#[test]
fn create_split_can_insert_before_target_tabs() {
    let manager = LayoutManager::default();
    let existing = ViewInstanceId::new("editor.scene#1");
    let inserted = ViewInstanceId::new("editor.hierarchy#1");
    let mut layout = WorkbenchLayout::default();

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: existing.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        )
        .unwrap();

    manager
        .apply(
            &mut layout,
            LayoutCommand::CreateSplit {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
                new_instance: inserted.clone(),
            },
        )
        .unwrap();

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };

    let crate::DocumentNode::SplitNode { first, second, .. } = document_workspace else {
        panic!("expected split root");
    };
    let crate::DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first tabs");
    };
    let crate::DocumentNode::Tabs(second_tabs) = second.as_ref() else {
        panic!("expected second tabs");
    };

    assert_eq!(first_tabs.tabs, vec![inserted]);
    assert_eq!(second_tabs.tabs, vec![existing]);
}

#[test]
fn set_drawer_extent_clamps_to_minimum_size() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: 48.0,
            },
        )
        .unwrap();

    assert_eq!(layout.drawers[&ActivityDrawerSlot::LeftTop].extent, 120.0);
}

#[test]
fn attach_view_to_same_drawer_reorders_it_to_the_end_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.project#1");
    let second = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();

    let drawer = layout.drawers.get(&ActivityDrawerSlot::RightTop).unwrap();
    assert_eq!(drawer.tab_stack.tabs, vec![second, first.clone()]);
    assert_eq!(drawer.tab_stack.active_tab.as_ref(), Some(&first));
    assert_eq!(drawer.active_view.as_ref(), Some(&first));
}

#[test]
fn attach_view_to_drawer_inserts_before_anchor_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.project#1");
    let second = ViewInstanceId::new("editor.inspector#1");
    let inserted = ViewInstanceId::new("editor.console#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: inserted.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: Some(TabInsertionAnchor {
                    target_id: second.clone(),
                    side: TabInsertionSide::Before,
                }),
            },
        )
        .unwrap();

    let drawer = layout.drawers.get(&ActivityDrawerSlot::RightBottom).unwrap();
    assert_eq!(drawer.tab_stack.tabs, vec![first, inserted.clone(), second]);
    assert_eq!(drawer.tab_stack.active_tab.as_ref(), Some(&inserted));
    assert_eq!(drawer.active_view.as_ref(), Some(&inserted));
}

#[test]
fn attach_view_to_document_inserts_after_anchor_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.scene#1");
    let second = ViewInstanceId::new("editor.game#1");
    let inserted = ViewInstanceId::new("editor.prefab#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: first.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: inserted.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
                anchor: Some(TabInsertionAnchor {
                    target_id: first.clone(),
                    side: TabInsertionSide::After,
                }),
            },
        )
        .unwrap();

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let crate::DocumentNode::Tabs(stack) = document_workspace else {
        panic!("expected tabs root");
    };

    assert_eq!(stack.tabs, vec![first, inserted.clone(), second]);
    assert_eq!(stack.active_tab.as_ref(), Some(&inserted));
}
