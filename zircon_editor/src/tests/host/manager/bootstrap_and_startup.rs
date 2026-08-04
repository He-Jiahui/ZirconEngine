use std::collections::BTreeMap;
use std::fs;

use crate::core::plugin::EditorPluginState;
use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, RecentProjectValidation,
    StoredRecentProjectEntry, StoredStartupSession,
};
use crate::ui::host::EditorManager;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, MainHostPageLayout, MainPageId,
    TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use zircon_runtime::core::manager::ManagerResolver;

use super::support::*;

#[test]
fn editor_manager_bootstrap_prefers_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global");
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
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
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
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
            ViewInstanceId::new("editor.hierarchy#1"),
            ViewInstanceId::new("editor.assets#1"),
        ]
    );
    assert_eq!(
        left_top.active_view,
        Some(ViewInstanceId::new("editor.hierarchy#1"))
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
fn opening_functional_editor_window_creates_instance_scoped_floating_window() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_functional_window_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.material_editor_window"), None)
        .unwrap();

    assert_eq!(
        instance_id,
        ViewInstanceId::new("editor.material_editor_window#1")
    );
    let window_id = MainPageId::new("window:editor.material_editor_window#1");
    let layout = manager.current_layout();
    let floating = layout
        .floating_windows
        .iter()
        .find(|window| window.window_id == window_id)
        .expect("material editor should open in a floating window");
    assert_eq!(floating.focused_view, Some(instance_id.clone()));
    assert!(floating.workspace.contains(&instance_id));
    let native_host = manager
        .native_window_hosts()
        .into_iter()
        .find(|host| host.window_id == window_id)
        .expect("floating editor window should own a native host state");
    assert_eq!(
        native_host.surface_tree_id.0,
        "zircon.editor.native_window.window:editor.material_editor_window#1"
    );
    assert_eq!(
        manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .map(|instance| instance.host),
        Some(ViewHost::FloatingWindow(window_id, vec![]))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn opening_drawer_backed_windows_creates_distinct_exclusive_pages() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_exclusive_window_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let asset_browser = manager
        .open_view(ViewDescriptorId::new("editor.asset_browser_window"), None)
        .unwrap();
    let diagnostics = manager
        .open_view(ViewDescriptorId::new("editor.diagnostics_window"), None)
        .unwrap();

    let asset_page = MainPageId::new("page:editor.asset_browser_window#1");
    let diagnostics_page = MainPageId::new("page:editor.diagnostics_window#1");
    let layout = manager.current_layout();
    assert!(layout
        .main_pages
        .iter()
        .any(|page| page.id() == &asset_page));
    assert!(layout
        .main_pages
        .iter()
        .any(|page| page.id() == &diagnostics_page));
    assert_eq!(layout.active_main_page, diagnostics_page);
    let instances = manager.current_view_instances();
    assert_eq!(
        instances
            .iter()
            .find(|instance| instance.instance_id == asset_browser)
            .map(|instance| instance.host.clone()),
        Some(ViewHost::ExclusivePage(asset_page))
    );
    assert_eq!(
        instances
            .iter()
            .find(|instance| instance.instance_id == diagnostics)
            .map(|instance| instance.host.clone()),
        Some(ViewHost::ExclusivePage(diagnostics_page))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
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
        focused_view: None,
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
        focused_view: Some(ViewInstanceId::new("editor.scene#1")),
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
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
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
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
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
    assert!(instances.contains(&ViewInstanceId::new("editor.assets#1")));
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
fn startup_session_defaults_to_component_showcase_without_recent_project() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_welcome");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert_eq!(
        session.open_builtin_view.as_deref(),
        Some("editor.ui_component_showcase")
    );
    assert!(session.project.is_none());
    assert!(session.recent_projects.is_empty());
    assert_eq!(session.draft.project_name, "ZirconProject");
    assert_eq!(session.draft.template, NewProjectTemplate::RenderableEmpty);
    assert_eq!(session.status_message, "Opened UI Component Showcase");

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
    let default_startup = manager.resolve_startup_session().unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert!(opened.open_builtin_view.is_none());
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].summary.name, "RecentProject");
    assert_eq!(recent[0].validation, RecentProjectValidation::Valid);
    assert_eq!(default_startup.mode, EditorSessionMode::Project);
    assert!(default_startup.project.is_some());
    assert!(default_startup.open_builtin_view.is_none());
    assert_eq!(default_startup.recent_projects.len(), 1);
    assert_eq!(
        default_startup.recent_projects[0].validation,
        RecentProjectValidation::Valid
    );
    assert!(default_startup
        .status_message
        .contains("Restored recent project"));
    assert!(default_startup
        .status_message
        .contains("RecentProject (scene="));
    assert!(opened
        .status_message
        .contains("scene=res://scenes/main.scene.toml"));
    assert!(default_startup
        .status_message
        .contains("scene=res://scenes/main.scene.toml"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(location);
}

#[test]
fn restored_legacy_manifest_recent_path_is_migrated_to_the_project_root() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_manifest_recent_migration");
    let project_root = unique_temp_dir("zircon_editor_startup_manifest_recent_project");
    create_project_with_default_world(&project_root);
    let manifest_path = project_root.join("zircon-project.toml");
    let manifest_path_string = manifest_path.to_string_lossy().into_owned();
    let summary = ProjectAuthority::default()
        .probe_project(&manifest_path)
        .unwrap()
        .summary()
        .clone();
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    let legacy_session = StoredStartupSession {
        last_project_path: Some(manifest_path_string.clone()),
        recent_projects: vec![StoredRecentProjectEntry {
            summary,
            path: manifest_path_string,
            last_opened_unix_ms: 42,
        }],
    };
    config
        .set_value(
            "editor.startup.session",
            serde_json::to_value(&legacy_session).unwrap(),
        )
        .unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let restored = manager.resolve_startup_session().unwrap();
    let saved = ProjectAuthority::default()
        .decode_startup_session(serde_json::Value::from(
            config
                .get_value("editor.startup.session")
                .expect("restored session must be saved"),
        ))
        .unwrap();
    let project_root = project_root.to_string_lossy().into_owned();

    assert_eq!(restored.mode, EditorSessionMode::Project);
    assert_eq!(restored.recent_projects.len(), 1);
    assert_eq!(restored.recent_projects[0].path, project_root);
    assert_eq!(
        saved.last_project_path.as_deref(),
        Some(project_root.as_str())
    );
    assert_eq!(saved.recent_projects.len(), 1);
    assert_eq!(saved.recent_projects[0].path, project_root);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn explicit_project_open_session_bypasses_component_showcase_builtin_view() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_project_open");
    let project_root = unique_temp_dir("zircon_editor_explicit_project_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let opened = manager.open_project_and_remember(&project_root).unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert!(opened.open_builtin_view.is_none());
    assert!(opened.status_message.starts_with("Project opened:"));
    assert!(opened.status_message.contains("assets="));
    assert_eq!(opened.recent_projects.len(), 1);
    assert_eq!(
        opened.recent_projects[0].validation,
        RecentProjectValidation::Valid
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn project_open_applies_completed_plugin_manifest_to_the_manager() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_project_plugin_manifest");
    let project_root = unique_temp_dir("zircon_editor_project_plugin_manifest");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let opened = manager.open_project_and_remember(&project_root).unwrap();
    let plugin_source = manager.plugin_panel_source();
    let plugin_rows = plugin_source.rows().collect::<Vec<_>>();

    assert!(opened.project.is_some());
    assert!(!plugin_rows.is_empty());
    assert!(
        plugin_rows
            .iter()
            .all(|row| row.state() == EditorPluginState::Disabled),
        "the default project manifest must disable every completed editor package"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn startup_session_falls_back_to_welcome_when_last_project_is_missing() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_missing_recent");
    let project_root = unique_temp_dir("zircon_editor_missing_recent_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project_and_remember(&project_root).unwrap();
    fs::remove_dir_all(&project_root).unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert_eq!(
        session.open_builtin_view.as_deref(),
        Some("editor.ui_component_showcase")
    );
    assert!(session.project.is_none());
    assert_eq!(session.recent_projects.len(), 1);
    assert_eq!(
        session.recent_projects[0].validation,
        RecentProjectValidation::Missing
    );
    assert!(session
        .status_message
        .contains("Could not restore last project"));
    assert!(session
        .status_message
        .contains("Opened UI Component Showcase"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn project_open_with_corrupt_workspace_falls_back_to_global_layout_with_diagnostic() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_corrupt_workspace_fallback");
    let project_root = unique_temp_dir("zircon_editor_corrupt_workspace_project");
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
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
    create_project_with_default_world(&project_root);
    let workspace_path = project_root.join(".zircon").join("editor-workspace.json");
    fs::create_dir_all(workspace_path.parent().unwrap()).unwrap();
    fs::write(&workspace_path, "{ this is not valid workspace json").unwrap();

    let opened = manager.open_project_and_remember(&project_root).unwrap();
    let project = opened.project.as_ref().expect("project should still open");

    assert!(project.editor_workspace.is_none());
    assert_eq!(project.workspace_restore_diagnostics.len(), 1);
    assert!(opened
        .status_message
        .contains("Project opened with default layout"));
    assert!(opened.status_message.contains("editor-workspace.json"));

    manager
        .apply_project_workspace(project.editor_workspace.clone())
        .unwrap();
    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("global-layout")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}
