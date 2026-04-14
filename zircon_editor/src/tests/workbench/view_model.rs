use std::collections::BTreeMap;

use zircon_math::UVec2;

use crate::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout,
    MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::snapshot::{
    AssetWorkspaceSnapshot, DocumentWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot,
    NewProjectFormSnapshot, ProjectOverviewSnapshot, RecentProjectItemSnapshot,
    WelcomePaneSnapshot,
};
use crate::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};
use crate::{
    default_preview_fixture, EditorSessionMode, MainHostStripModel, MenuAction,
    RecentProjectValidation, ViewContentKind,
    ViewportTextureBridge, ViewportTextureBridgeError, WorkbenchViewModel,
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
        .any(|item| item.action == MenuAction::Undo && item.enabled));
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
fn viewport_texture_bridge_rejects_invalid_metadata() {
    assert_eq!(
        ViewportTextureBridge::validate_metadata(
            0,
            720,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        ),
        Err(ViewportTextureBridgeError::UnsupportedFormat(
            wgpu::TextureFormat::Rgba16Float
        ))
    );
    assert_eq!(
        ViewportTextureBridge::validate_metadata(
            1280,
            720,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING,
        ),
        Err(ViewportTextureBridgeError::MissingUsage(
            wgpu::TextureUsages::RENDER_ATTACHMENT
        ))
    );
}

#[test]
fn default_preview_fixture_projects_drawers_and_document_workspace() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    let left_top = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Project));
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Assets));
    assert!(left_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Hierarchy));

    let right_top = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert!(right_top
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Inspector));

    let bottom_left = model
        .drawer_ring
        .drawers
        .get(&ActivityDrawerSlot::BottomLeft)
        .expect("bottom left drawer");
    assert!(bottom_left
        .tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::Console));

    match &model.document {
        crate::DocumentWorkspaceModel::Workbench { workspace, .. } => match workspace {
            DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
                assert!(tabs
                    .iter()
                    .any(|tab| tab.content_kind == ViewContentKind::Scene));
                assert!(tabs
                    .iter()
                    .any(|tab| tab.content_kind == ViewContentKind::Game));
                assert_eq!(
                    active_tab.as_ref().map(|id| id.0.as_str()),
                    Some("editor.scene#1")
                );
            }
            DocumentWorkspaceSnapshot::Split { .. } => {
                panic!("preview fixture should use tab workspace")
            }
        },
        crate::DocumentWorkspaceModel::Exclusive { .. } => {
            panic!("preview fixture should use workbench page")
        }
    }
}

#[test]
fn default_preview_fixture_exposes_hybrid_shell_tool_windows_and_empty_states() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let model = WorkbenchViewModel::build(&chrome);

    let left_top = model
        .tool_windows
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top tool window");
    assert_eq!(left_top.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        left_top
            .tabs
            .iter()
            .map(|tab| tab.content_kind)
            .collect::<Vec<_>>(),
        vec![
            ViewContentKind::Project,
            ViewContentKind::Assets,
            ViewContentKind::Hierarchy,
        ]
    );
    assert_eq!(
        left_top.active_tab.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let project_tab = left_top
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Project)
        .expect("project tab");
    assert!(!project_tab.closeable);
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .and_then(|state| state.primary_action.as_ref())
            .map(|action| action.label.as_str()),
        Some("Open Project")
    );

    let right_top = model
        .tool_windows
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top tool window");
    assert_eq!(right_top.mode, ActivityDrawerMode::Pinned);
    let inspector_tab = right_top
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Inspector)
        .expect("inspector tab");
    assert_eq!(
        inspector_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("Nothing selected")
    );

    let bottom_left = model
        .tool_windows
        .get(&ActivityDrawerSlot::BottomLeft)
        .expect("bottom left tool window");
    let console_tab = bottom_left
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Console)
        .expect("console tab");
    assert_eq!(
        console_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No output yet")
    );

    assert_eq!(
        model
            .document_tabs
            .iter()
            .map(|tab| tab.content_kind)
            .collect::<Vec<_>>(),
        vec![ViewContentKind::Scene, ViewContentKind::Game]
    );
    assert!(model.document_tabs.iter().all(|tab| !tab.closeable));
    assert!(!model
        .document_tabs
        .iter()
        .any(|tab| tab.content_kind == ViewContentKind::PrefabEditor));

    let scene_tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Scene)
        .expect("scene tab");
    assert_eq!(
        scene_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
}

#[test]
fn project_empty_state_remains_the_same_when_docked_to_the_right() {
    let mut fixture = default_preview_fixture();
    let left_top = fixture
        .layout
        .drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    left_top
        .tab_stack
        .tabs
        .retain(|instance_id| instance_id.0 != "editor.project#1");
    left_top.tab_stack.active_tab = Some(crate::ViewInstanceId::new("editor.assets#1"));
    left_top.active_view = Some(crate::ViewInstanceId::new("editor.assets#1"));

    let right_top = fixture
        .layout
        .drawers
        .get_mut(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    right_top
        .tab_stack
        .tabs
        .insert(0, crate::ViewInstanceId::new("editor.project#1"));
    right_top.tab_stack.active_tab = Some(crate::ViewInstanceId::new("editor.project#1"));
    right_top.active_view = Some(crate::ViewInstanceId::new("editor.project#1"));
    right_top.mode = ActivityDrawerMode::Pinned;

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);

    let project_tab = model
        .tool_windows
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top tool window")
        .tabs
        .iter()
        .find(|tab| tab.content_kind == ViewContentKind::Project)
        .expect("project tab");
    assert_eq!(
        project_tab
            .empty_state
            .as_ref()
            .map(|state| state.title.as_str()),
        Some("No project open")
    );
}

#[test]
fn welcome_startup_projects_into_exclusive_page_model() {
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.welcome"),
        ViewKind::ActivityWindow,
        "Welcome",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_icon_key("welcome")];
    let welcome_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.welcome#1"),
        descriptor_id: ViewDescriptorId::new("editor.welcome"),
        title: "Welcome".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("page:welcome")),
    };
    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Welcome".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Welcome,
            welcome: WelcomePaneSnapshot {
                title: "Open or Create".to_string(),
                subtitle: "Recent projects and a renderable empty-project template".to_string(),
                status_message: "No recent project".to_string(),
                browse_supported: false,
                recent_projects: vec![RecentProjectItemSnapshot {
                    display_name: "Broken".to_string(),
                    path: "E:/Missing/Broken".to_string(),
                    validation: RecentProjectValidation::Missing,
                    last_opened_label: "Just now".to_string(),
                    selected: true,
                }],
                form: NewProjectFormSnapshot {
                    project_name: "WelcomeProject".to_string(),
                    location: "E:/Work".to_string(),
                    project_path_preview: "E:/Work/WelcomeProject".to_string(),
                    template_label: "Renderable Empty".to_string(),
                    can_create: true,
                    can_open_existing: true,
                    validation_message: String::new(),
                },
            },
            project_open: false,
            can_undo: false,
            can_redo: false,
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::new("page:welcome"),
            main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:welcome"),
                title: "Welcome".to_string(),
                window_instance: welcome_instance.instance_id.clone(),
            }],
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![welcome_instance],
        descriptors,
    );

    let model = WorkbenchViewModel::build(&chrome);

    assert!(!model.drawer_ring.visible);
    assert!(matches!(
        model.document,
        crate::DocumentWorkspaceModel::Exclusive { ref view, .. }
            if view.content_kind == ViewContentKind::Welcome
    ));
    assert_eq!(
        model
            .host_strip
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Welcome", "Open or Create"]
    );
}

fn sample_workbench_chrome() -> EditorChromeSnapshot {
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#1"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/main.scene" }),
        dirty: false,
        host: ViewHost::Document(MainPageId::workbench(), vec![]),
    };
    let hierarchy_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#1"),
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

fn sample_exclusive_chrome() -> EditorChromeSnapshot {
    let prefab_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.prefab#1"),
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
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![prefab_instance],
        descriptors,
    )
}
