use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::core::manager::resolve_config_manager;
use zircon_runtime::scene::DefaultLevelManager;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, LayoutCommand, MainHostPageLayout,
    MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::project::{EditorProjectDocument, ProjectEditorWorkspace};
use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectDraft, NewProjectTemplate, RecentProjectValidation,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::support::*;

#[test]
fn editor_manager_bootstrap_prefers_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global");
    let runtime = editor_runtime_with_config_path(&path);
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    let custom_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&custom_layout).unwrap(),
        )
        .unwrap();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        custom_layout.active_main_page
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn editor_manager_bootstrap_repairs_empty_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global_empty");
    let runtime = editor_runtime_with_config_path(&path);
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    let empty_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&empty_layout).unwrap(),
        )
        .unwrap();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let layout = manager.current_layout();

    assert_eq!(layout.active_main_page, MainPageId::new("global-layout"));

    let left_top = layout
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert_eq!(
        left_top.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.project#1"),
            ViewInstanceId::new("editor.assets#1"),
            ViewInstanceId::new("editor.hierarchy#1"),
        ]
    );
    assert_eq!(
        left_top.active_view,
        Some(ViewInstanceId::new("editor.project#1"))
    );

    let right_top = layout
        .drawers
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom = layout
        .drawers
        .get(&ActivityDrawerSlot::Bottom)
        .expect("bottom drawer");
    assert_eq!(
        bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );

    let workbench_page = layout
        .main_pages
        .iter()
        .find_map(|page| match page {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
        })
        .expect("workbench page");
    let DocumentNode::Tabs(document_tabs) = workbench_page else {
        panic!("expected root document tabs");
    };
    assert_eq!(
        document_tabs.tabs,
        vec![
            ViewInstanceId::new("editor.scene#1"),
            ViewInstanceId::new("editor.game#1"),
        ]
    );
    assert_eq!(
        document_tabs.active_tab,
        Some(ViewInstanceId::new("editor.scene#1"))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn save_and_load_preset_roundtrip_through_manager_commands() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager
        .apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::SavePreset {
            name: "authoring".to_string(),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::LoadPreset {
            name: "authoring".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn save_and_load_preset_roundtrip_through_project_asset_files() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project_presets");
    let project_root = unique_temp_dir("zircon_editor_project_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();
    manager.open_project(&project_root).unwrap();

    manager
        .apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
        .unwrap();

    let preset_asset = project_root
        .join("assets")
        .join("editor")
        .join("layout-presets")
        .join("rider.workbench-layout.json");
    assert!(
        preset_asset.exists(),
        "expected preset asset at {:?}",
        preset_asset
    );

    manager
        .apply_layout_command(LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn applying_project_workspace_restores_single_instance_registry_state() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let restored_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#restored"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    };
    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: DocumentNode::default(),
            }],
            drawers: BTreeMap::from([(
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![restored_instance.instance_id.clone()],
                        active_tab: Some(restored_instance.instance_id.clone()),
                    },
                    active_view: Some(restored_instance.instance_id.clone()),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 260.0,
                    visible: true,
                },
            )]),
            activity_windows: BTreeMap::from([(
                ActivityWindowId::workbench(),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::workbench(),
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::new(),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            )]),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        open_view_instances: vec![restored_instance.clone()],
        active_center_tab: None,
        active_drawers: vec![ActivityDrawerSlot::LeftTop],
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let layout = manager.current_layout();
    let left_top = layout
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tab_stack
        .tabs
        .contains(&restored_instance.instance_id));
    assert!(!left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
    let activity_left_top = layout
        .activity_windows
        .get(&ActivityWindowId::workbench())
        .and_then(|window| window.activity_drawers.get(&ActivityDrawerSlot::LeftTop))
        .expect("workbench activity left top drawer");
    assert!(activity_left_top
        .tab_stack
        .tabs
        .contains(&restored_instance.instance_id));

    let reopened = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();

    assert_eq!(reopened, restored_instance.instance_id);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn applying_project_workspace_preserves_builtin_shell_drawers() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project_shell_drawers");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![
                        ViewInstanceId::new("editor.scene#1"),
                        ViewInstanceId::new("editor.game#1"),
                    ],
                    active_tab: Some(ViewInstanceId::new("editor.scene#1")),
                }),
            }],
            drawers: BTreeMap::from([(
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftTop,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 0.0,
                    visible: false,
                },
            )]),
            activity_windows: BTreeMap::from([(
                ActivityWindowId::workbench(),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::workbench(),
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::from([(
                        ActivityDrawerSlot::BottomLeft,
                        ActivityDrawerLayout {
                            slot: ActivityDrawerSlot::BottomLeft,
                            tab_stack: TabStackLayout::default(),
                            active_view: None,
                            mode: ActivityDrawerMode::Collapsed,
                            extent: 0.0,
                            visible: false,
                        },
                    )]),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            )]),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        open_view_instances: vec![
            ViewInstance {
                instance_id: ViewInstanceId::new("editor.scene#1"),
                descriptor_id: ViewDescriptorId::new("editor.scene"),
                title: "Scene".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
            ViewInstance {
                instance_id: ViewInstanceId::new("editor.game#1"),
                descriptor_id: ViewDescriptorId::new("editor.game"),
                title: "Game".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        ],
        active_center_tab: Some(ViewInstanceId::new("editor.scene#1")),
        active_drawers: Vec::new(),
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let layout = manager.current_layout();

    let left_top = layout
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.project#1")));
    assert_eq!(left_top.mode, ActivityDrawerMode::Pinned);
    assert!(left_top.visible);
    assert!(left_top.extent > 0.0);

    let right_top = layout
        .drawers
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom = layout
        .drawers
        .get(&ActivityDrawerSlot::Bottom)
        .expect("bottom drawer");
    assert_eq!(
        bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );

    let workbench_window = layout
        .activity_windows
        .get(&ActivityWindowId::workbench())
        .expect("workbench activity window");
    let activity_left_top = workbench_window
        .activity_drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("workbench activity left top drawer");
    assert!(activity_left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.project#1")));
    let activity_bottom = workbench_window
        .activity_drawers
        .get(&ActivityDrawerSlot::Bottom)
        .expect("workbench activity bottom drawer");
    assert_eq!(
        activity_bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );
    assert_eq!(activity_bottom.mode, ActivityDrawerMode::Pinned);
    assert!(activity_bottom.visible);
    assert!(activity_bottom.extent > 0.0);

    let instances = manager
        .current_view_instances()
        .into_iter()
        .map(|instance| instance.instance_id)
        .collect::<Vec<_>>();
    assert!(instances.contains(&ViewInstanceId::new("editor.project#1")));
    assert!(instances.contains(&ViewInstanceId::new("editor.inspector#1")));
    assert!(instances.contains(&ViewInstanceId::new("editor.console#1")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn scene_and_game_tabs_are_not_closeable() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_non_closeable_docs");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(!manager
        .close_view(&ViewInstanceId::new("editor.scene#1"))
        .unwrap());
    assert!(!manager
        .close_view(&ViewInstanceId::new("editor.game#1"))
        .unwrap());
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id.0 == "editor.scene#1"));
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id.0 == "editor.game#1"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn editor_manager_registers_animation_document_view_descriptors() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_animation_view_descriptors");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let descriptor_ids = manager
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.descriptor_id)
        .collect::<Vec<_>>();

    assert!(descriptor_ids.contains(&ViewDescriptorId::new("editor.animation_sequence")));
    assert!(descriptor_ids.contains(&ViewDescriptorId::new("editor.animation_graph")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn startup_session_defaults_to_welcome_without_recent_project() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_welcome");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert!(session.project.is_none());
    assert!(session.recent_projects.is_empty());
    assert_eq!(session.draft.project_name, "ZirconProject");
    assert_eq!(session.draft.template, NewProjectTemplate::RenderableEmpty);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn create_project_and_open_persists_recent_project_and_returns_project_session() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_recent");
    let location = unique_temp_dir("zircon_editor_welcome_recent");
    fs::create_dir_all(&location).unwrap();
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let draft = NewProjectDraft {
        project_name: "RecentProject".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let opened = manager.create_project_and_open(draft).unwrap();
    let recent = manager.recent_projects_snapshot().unwrap();
    let reopened = manager.resolve_startup_session().unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].display_name, "RecentProject");
    assert_eq!(recent[0].validation, RecentProjectValidation::Valid);
    assert_eq!(reopened.mode, EditorSessionMode::Project);
    assert!(reopened.project.is_some());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(location);
}
