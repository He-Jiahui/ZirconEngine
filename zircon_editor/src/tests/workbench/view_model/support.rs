use std::collections::BTreeMap;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DocumentNode,
    FloatingWindowLayout, MainHostPageLayout, MainPageId, SplitAxis, TabStackLayout,
    WorkbenchLayout,
};
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewKind,
};
use zircon_runtime_interface::math::UVec2;

pub(super) fn sample_workbench_chrome() -> EditorChromeSnapshot {
    let scene_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.scene#1"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/main.scene" }),
        dirty: false,
        host: ViewHost::Document(MainPageId::workbench(), vec![]),
    };
    let hierarchy_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.hierarchy#1"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    };
    let descriptors = vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.scene"),
            ViewKind::ActivityView,
            "Scene",
        )
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_icon_key("scene"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_icon_key("hierarchy"),
    ];
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            activity_window: ActivityWindowId::workbench(),
            document_workspace: DocumentNode::Tabs(TabStackLayout {
                tabs: vec![scene_instance.instance_id.clone()],
                active_tab: Some(scene_instance.instance_id.clone()),
            }),
        }],
        drawers: BTreeMap::from([(
            ActivityDrawerSlot::LeftTop,
            ActivityDrawerLayout {
                slot: ActivityDrawerSlot::LeftTop,
                tab_stack: TabStackLayout {
                    tabs: vec![hierarchy_instance.instance_id.clone()],
                    active_tab: Some(hierarchy_instance.instance_id.clone()),
                },
                active_view: Some(hierarchy_instance.instance_id.clone()),
                mode: ActivityDrawerMode::Pinned,
                extent: 288.0,
                visible: true,
            },
        )]),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Editor booted".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: SceneViewportSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: "sandbox-project".to_string(),
            session_mode: EditorSessionMode::Project,
            welcome: WelcomePaneSnapshot::default(),
            project_open: true,
            can_undo: true,
            can_redo: false,
        },
        &layout,
        vec![scene_instance, hierarchy_instance],
        descriptors,
    )
}

pub(super) fn sample_exclusive_chrome() -> EditorChromeSnapshot {
    let prefab_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.prefab#1"),
        descriptor_id: ViewDescriptorId::new("editor.prefab"),
        title: "Prefab Editor".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://player.prefab" }),
        dirty: true,
        host: ViewHost::ExclusivePage(MainPageId::new("page:prefab")),
    };
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.prefab"),
        ViewKind::ActivityWindow,
        "Prefab Editor",
    )
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_icon_key("prefab")];
    EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Prefab mode".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1024, 768),
            scene_viewport_settings: SceneViewportSettings::default(),
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
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::new("page:prefab"),
            main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:prefab"),
                title: "Prefab Editor".to_string(),
                window_instance: prefab_instance.instance_id.clone(),
            }],
            drawers: BTreeMap::from([(
                ActivityDrawerSlot::RightTop,
                ActivityDrawerLayout::new(ActivityDrawerSlot::RightTop),
            )]),
            activity_windows: Default::default(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![prefab_instance],
        descriptors,
    )
}

pub(super) fn sample_floating_window_chrome() -> EditorChromeSnapshot {
    let scene_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.scene#1"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/main.scene" }),
        dirty: false,
        host: ViewHost::Document(MainPageId::workbench(), vec![]),
    };
    let floating_scene_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.scene#float"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(MainPageId::new("window:prefab"), vec![0]),
    };
    let prefab_instance = ViewInstance {
        instance_id: crate::ui::workbench::view::ViewInstanceId::new("editor.prefab#float"),
        descriptor_id: ViewDescriptorId::new("editor.prefab"),
        title: "Prefab Editor".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://player.prefab" }),
        dirty: true,
        host: ViewHost::FloatingWindow(MainPageId::new("window:prefab"), vec![1]),
    };
    let descriptors = vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.scene"),
            ViewKind::ActivityView,
            "Scene",
        )
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_icon_key("scene"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.prefab"),
            ViewKind::ActivityWindow,
            "Prefab Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_icon_key("prefab"),
    ];

    EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Floating prefab".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: SceneViewportSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: "sandbox-project".to_string(),
            session_mode: EditorSessionMode::Project,
            welcome: WelcomePaneSnapshot::default(),
            project_open: true,
            can_undo: true,
            can_redo: true,
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![scene_instance.instance_id.clone()],
                    active_tab: Some(scene_instance.instance_id.clone()),
                }),
            }],
            drawers: BTreeMap::from([(
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop),
            )]),
            activity_windows: Default::default(),
            floating_windows: vec![FloatingWindowLayout {
                window_id: MainPageId::new("window:prefab"),
                title: "Prefab Popout".to_string(),
                workspace: DocumentNode::SplitNode {
                    axis: SplitAxis::Horizontal,
                    ratio: 0.5,
                    first: Box::new(DocumentNode::Tabs(TabStackLayout {
                        tabs: vec![floating_scene_instance.instance_id.clone()],
                        active_tab: Some(floating_scene_instance.instance_id.clone()),
                    })),
                    second: Box::new(DocumentNode::Tabs(TabStackLayout {
                        tabs: vec![prefab_instance.instance_id.clone()],
                        active_tab: Some(prefab_instance.instance_id.clone()),
                    })),
                },
                focused_view: Some(prefab_instance.instance_id.clone()),
                frame: ShellFrame::new(111.0, 92.0, 640.0, 420.0),
            }],
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![scene_instance, floating_scene_instance, prefab_instance],
        descriptors,
    )
}
