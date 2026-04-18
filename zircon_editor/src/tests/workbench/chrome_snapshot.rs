use std::collections::BTreeMap;

use zircon_math::UVec2;

use crate::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout,
    MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::snapshot::{
    AssetWorkspaceSnapshot, DocumentWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot,
    MainPageSnapshot, ProjectOverviewSnapshot,
};
use crate::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use zircon_scene::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportSettings, SceneViewportTool,
    TransformSpace, ViewOrientation,
};

#[test]
fn chrome_builder_keeps_placeholder_tabs_for_missing_view_instances() {
    let present = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#1"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    };
    let missing = ViewInstanceId::new("editor.scene#missing");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            document_workspace: DocumentNode::Tabs(TabStackLayout {
                tabs: vec![missing.clone()],
                active_tab: Some(missing.clone()),
            }),
        }],
        drawers: BTreeMap::from([(
            ActivityDrawerSlot::LeftTop,
            ActivityDrawerLayout {
                slot: ActivityDrawerSlot::LeftTop,
                tab_stack: TabStackLayout {
                    tabs: vec![present.instance_id.clone()],
                    active_tab: Some(present.instance_id.clone()),
                },
                active_view: Some(present.instance_id.clone()),
                mode: ActivityDrawerMode::Pinned,
                extent: 260.0,
                visible: true,
            },
        )]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.hierarchy"),
        ViewKind::ActivityView,
        "Hierarchy",
    )
    .with_preferred_host(PreferredHost::Drawer(ActivityDrawerSlot::LeftTop))];

    let chrome = EditorChromeSnapshot::build(
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
        },
        &layout,
        vec![present],
        descriptors,
    );

    let drawer = chrome
        .workbench
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .unwrap();
    assert_eq!(drawer.tabs.len(), 1);
    assert!(!drawer.tabs[0].placeholder);

    let MainPageSnapshot::Workbench { workspace, .. } = &chrome.workbench.main_pages[0] else {
        panic!("expected workbench page");
    };
    let DocumentWorkspaceSnapshot::Tabs { tabs, .. } = workspace else {
        panic!("expected tabs root");
    };
    assert_eq!(tabs.len(), 1);
    assert!(tabs[0].placeholder);
    assert!(tabs[0].title.contains("Missing"));
}

#[test]
fn chrome_builder_marks_exclusive_activity_window_pages() {
    let asset_browser = ViewInstance {
        instance_id: ViewInstanceId::new("editor.asset_browser#1"),
        descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
        title: "Asset Browser".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("asset-browser")),
    };
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::new("asset-browser"),
        main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
            id: MainPageId::new("asset-browser"),
            title: "Asset Browser".to_string(),
            window_instance: asset_browser.instance_id.clone(),
        }],
        drawers: BTreeMap::new(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.asset_browser"),
        ViewKind::ActivityWindow,
        "Asset Browser",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)];

    let chrome = EditorChromeSnapshot::build(
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
        },
        &layout,
        vec![asset_browser.clone()],
        descriptors,
    );

    assert_eq!(
        chrome.workbench.active_main_page,
        MainPageId::new("asset-browser")
    );
    let MainPageSnapshot::Exclusive { view, .. } = &chrome.workbench.main_pages[0] else {
        panic!("expected exclusive page");
    };
    assert_eq!(view.instance_id, asset_browser.instance_id);
    assert!(!view.placeholder);
}

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
