use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_event::{MenuAction, ViewDescriptorId as EventViewDescriptorId};
use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::workbench::autolayout::ShellFrame;
use std::collections::BTreeMap;

use crate::scene::viewport::SceneViewportChromeSettings;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, MainHostPageLayout, MainPageId,
    TabStackLayout, WorkbenchLayout, WorkspaceTarget,
};
use crate::ui::workbench::model::{MainHostStripModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
    StatusTaskProgressSnapshot, StatusTaskProgressTone,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind,
};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::UVec2;

use super::support::{
    sample_exclusive_chrome, sample_floating_window_chrome, sample_workbench_chrome,
};

#[test]
fn workbench_view_model_projects_menu_strip_drawers_and_status() {
    let chrome = sample_workbench_chrome();

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    assert_eq!(
        model
            .menu_bar
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect::<Vec<_>>(),
        vec![
            "File",
            "Edit",
            "Selection",
            "Play",
            "View",
            "Window",
            "Help"
        ]
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
    assert_eq!(undo_operation, "edit.history.undo");
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
    assert_eq!(model.status_bar.error_text, "No Errors");
    assert_eq!(model.status_bar.warning_text, "0 Warnings");
    assert_eq!(model.status_bar.message_text, "0 Messages");
    assert_eq!(model.status_bar.grid_text, "Grid: 1 m");
    assert_eq!(model.status_bar.snap_text, "Snap: Off");
    assert_eq!(model.status_bar.zoom_text, "100%");
    assert!(model.status_bar.task_progress.is_none());
    let save_project_binding = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "File")
        .and_then(|menu| menu.items.iter().find(|item| item.label == "Save Project"))
        .map(|item| crate::ui::workbench::event::menu_item_binding(item).native_binding())
        .expect("save project binding");
    assert_eq!(
        save_project_binding,
        r#"WorkbenchMenuBar/SaveProject:onClick(MenuAction("workbench.project.save"))"#
    );
    let selection_menu = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Selection")
        .expect("selection menu");
    assert_eq!(
        selection_menu
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Create Cube",
            "Create Camera",
            "Create Ambient Light",
            "Create Directional Light",
            "Create Point Light",
            "Create Rect Light",
            "Create Spot Light",
            "Delete Selection",
        ]
    );
    let rect_light_item = selection_menu
        .items
        .iter()
        .find(|item| item.label == "Create Rect Light")
        .expect("rect light create menu item");
    assert_eq!(
        rect_light_item.action,
        Some(MenuAction::CreateNode(NodeKind::RectLight))
    );
    assert_eq!(
        rect_light_item
            .operation_path
            .as_ref()
            .map(|path| path.as_str()),
        Some("scene.node.create_rect_light")
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
    assert_eq!(reset_layout_operation, "window.layout.reset");
    let debug_observatory_item = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Window")
        .and_then(|menu| {
            menu.items
                .iter()
                .find(|item| item.label == "Debug Observatory")
        })
        .expect("debug observatory window menu item");
    assert_eq!(
        debug_observatory_item.action,
        Some(MenuAction::OpenView(EventViewDescriptorId::new(
            "editor.debug_observatory"
        )))
    );
    assert_eq!(
        debug_observatory_item
            .operation_path
            .as_ref()
            .map(|path| path.as_str()),
        Some("window.debug_observatory.open")
    );
}

#[test]
fn workbench_view_model_projects_status_task_progress_slot() {
    let mut chrome = sample_workbench_chrome();
    chrome.status_task_progress = Some(
        StatusTaskProgressSnapshot::new("desktop_export:3", "Export desktop_linux")
            .with_detail("cargo-build - Running generated build")
            .with_percent(64)
            .with_tone(StatusTaskProgressTone::Info),
    );

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    assert_eq!(model.status_bar.message_text, "1 Message");
    let task = model
        .status_bar
        .task_progress
        .expect("status task should be projected");
    assert_eq!(task.task_id, "desktop_export:3");
    assert_eq!(task.label, "Export desktop_linux");
    assert_eq!(task.percent, Some(64));
}

#[test]
fn workbench_window_menu_exposes_unreal_style_functional_windows() {
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &sample_workbench_chrome(),
    );
    let window_menu = model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Window")
        .expect("window menu");

    for (label, descriptor_id, operation_path) in [
        (
            "Prefab Editor",
            "editor.prefab_editor_window",
            "window.prefab_editor.open",
        ),
        (
            "Material Editor",
            "editor.material_editor_window",
            "window.material_editor.open",
        ),
        (
            "UI Component Showcase",
            "editor.ui_component_showcase",
            "window.ui_component_showcase.open",
        ),
        (
            "Material Demo",
            "editor.material_demo_window",
            "window.material_demo.open",
        ),
        (
            "UI Asset Editor",
            "editor.ui_asset_editor_window",
            "window.ui_asset_editor.open",
        ),
        (
            "Animation Editor",
            "editor.animation_editor_window",
            "window.animation_editor.open",
        ),
        (
            "Asset Browser",
            "editor.asset_browser_window",
            "window.asset_browser.open",
        ),
        (
            "Diagnostics",
            "editor.diagnostics_window",
            "window.diagnostics.open",
        ),
    ] {
        let item = window_menu
            .items
            .iter()
            .find(|item| item.label == label)
            .unwrap_or_else(|| panic!("missing `{label}` window menu item"));
        assert_eq!(
            item.action,
            Some(MenuAction::OpenView(EventViewDescriptorId::new(
                descriptor_id
            )))
        );
        assert_eq!(
            item.operation_path.as_ref().map(|path| path.as_str()),
            Some(operation_path)
        );
        assert_eq!(
            crate::ui::workbench::event::menu_item_binding(item).native_binding(),
            format!(
                r#"WorkbenchMenuBar/OpenView.{descriptor_id}:onClick(MenuAction("workbench.view.open.{descriptor_id}"))"#
            )
        );
    }
}

#[test]
fn workbench_view_model_freezes_drawers_for_exclusive_page() {
    let chrome = sample_exclusive_chrome();

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

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
fn workbench_view_model_uses_only_active_activity_window_drawers() {
    let chrome =
        sample_two_activity_windows_chrome(ActivityWindowId::new("window:z_asset_browser"));

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    assert!(chrome.workbench.drawers.is_empty());
    assert!(!model.drawer_ring.visible);
    assert!(model.drawer_ring.drawers.is_empty());
    assert!(model.tool_windows.is_empty());
}

#[test]
fn workbench_view_model_uses_active_activity_window_drawer_extent() {
    let chrome = sample_two_activity_windows_chrome(ActivityWindowId::new("window:workbench"));

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let drawer = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("active workbench window drawer");

    assert!(model.drawer_ring.visible);
    assert_eq!(drawer.extent, 288.0);
    assert_eq!(drawer.mode, ActivityDrawerMode::Pinned);
}

#[test]
fn workbench_view_model_exposes_floating_windows_as_workspace_tabs() {
    let chrome = sample_floating_window_chrome();

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

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

#[test]
fn workbench_view_model_filters_and_orders_plugin_menu_contributions() {
    let chrome = sample_workbench_chrome();
    let weather_capability = "editor.extension.weather_menu";
    let public_operation = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let gated_operation = EditorOperationPath::parse("weather.cloud_layer.secret").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                "Tools/Weather/Refresh Cloud Layers",
                public_operation.clone(),
            )
            .with_priority(20)
            .with_shortcut("Ctrl+Alt+R")
            .with_enabled(false),
        )
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                "Tools/Weather/Secret Cloud Pass",
                gated_operation.clone(),
            )
            .with_priority(-10)
            .with_shortcut("Ctrl+Alt+S")
            .with_required_capabilities([weather_capability]),
        )
        .unwrap();

    let mut commands = crate::core::commands::EditorCommandRegistry::default_workbench();
    commands
        .register(EditorCommandDescriptor::operation(
            public_operation,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    commands
        .register(
            EditorCommandDescriptor::operation(gated_operation, "Secret Cloud Pass")
                .with_required_capabilities([weather_capability]),
        )
        .unwrap();

    let disabled_model = WorkbenchViewModel::build_with_extensions_and_capabilities(
        &commands,
        &chrome,
        &[extension.clone()],
        &[],
    );
    let disabled_tools = disabled_model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Tools")
        .expect("tools menu");
    assert_eq!(
        disabled_tools
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Weather"]
    );
    assert_eq!(
        disabled_tools.items[0]
            .children
            .first()
            .and_then(|item| item.shortcut.as_deref()),
        Some("Ctrl+Alt+S")
    );
    assert!(!disabled_tools.items[0].enabled);
    assert_eq!(
        disabled_tools.items[0]
            .children
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Secret Cloud Pass", "Refresh Cloud Layers"]
    );

    let enabled_capabilities = vec![weather_capability.to_string()];
    let enabled_model = WorkbenchViewModel::build_with_extensions_and_capabilities(
        &commands,
        &chrome,
        &[extension],
        &enabled_capabilities,
    );
    let enabled_tools = enabled_model
        .menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == "Tools")
        .expect("tools menu");
    assert_eq!(
        enabled_tools.items[0]
            .children
            .iter()
            .map(|item| (item.label.as_str(), item.shortcut.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("Secret Cloud Pass", Some("Ctrl+Alt+S")),
            ("Refresh Cloud Layers", Some("Ctrl+Alt+R")),
        ]
    );
}

fn sample_two_activity_windows_chrome(active_window: ActivityWindowId) -> EditorChromeSnapshot {
    let workbench_page_id = MainPageId::workbench();
    let asset_page_id = MainPageId::new("asset-browser");
    let active_main_page = if active_window.as_str() == "window:workbench" {
        workbench_page_id.clone()
    } else {
        asset_page_id.clone()
    };

    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#1"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Document(workbench_page_id.clone(), vec![]),
    };
    let hierarchy_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#1"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    };
    let asset_browser_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.asset_browser#1"),
        descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
        title: "Asset Browser".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Document(asset_page_id.clone(), vec![]),
    };

    let workbench_drawer = ActivityDrawerLayout {
        slot: ActivityDrawerSlot::LeftTop,
        tab_stack: TabStackLayout {
            tabs: vec![hierarchy_instance.instance_id.clone()],
            active_tab: Some(hierarchy_instance.instance_id.clone()),
        },
        active_view: Some(hierarchy_instance.instance_id.clone()),
        mode: ActivityDrawerMode::Pinned,
        extent: 288.0,
        visible: true,
    };
    let mut stale_root_drawer = workbench_drawer.clone();
    stale_root_drawer.extent = 416.0;

    let layout = WorkbenchLayout {
        active_main_page,
        main_pages: vec![
            MainHostPageLayout::WorkbenchPage {
                id: workbench_page_id.clone(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::new("window:workbench"),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![scene_instance.instance_id.clone()],
                    active_tab: Some(scene_instance.instance_id.clone()),
                }),
            },
            MainHostPageLayout::WorkbenchPage {
                id: asset_page_id.clone(),
                title: "Asset Browser".to_string(),
                activity_window: ActivityWindowId::new("window:z_asset_browser"),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![asset_browser_instance.instance_id.clone()],
                    active_tab: Some(asset_browser_instance.instance_id.clone()),
                }),
            },
        ],
        drawers: BTreeMap::from([(ActivityDrawerSlot::LeftTop, stale_root_drawer)]),
        activity_windows: BTreeMap::from([
            (
                ActivityWindowId::new("window:workbench"),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::new("window:workbench"),
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::from([(
                        ActivityDrawerSlot::LeftTop,
                        workbench_drawer,
                    )]),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            ),
            (
                ActivityWindowId::new("window:z_asset_browser"),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::new("window:z_asset_browser"),
                    descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::new(),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            ),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Ready".to_string(),
            status_task_progress: None,
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: SceneViewportChromeSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: "sandbox-project".to_string(),
            session_mode: EditorSessionMode::Project,
            welcome: WelcomePaneSnapshot::default(),
            project_open: true,
            can_undo: false,
            can_redo: false,
            bridge_diagnostics: Default::default(),
        },
        &layout,
        vec![scene_instance, hierarchy_instance, asset_browser_instance],
        vec![
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.scene"),
                ViewKind::ActivityView,
                "Scene",
            ),
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.hierarchy"),
                ViewKind::ActivityView,
                "Hierarchy",
            ),
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.asset_browser"),
                ViewKind::ActivityWindow,
                "Asset Browser",
            ),
        ],
        None,
    )
}
