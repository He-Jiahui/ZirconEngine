use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::CoreRuntime;
use zircon_manager::{resolve_config_manager, MANAGER_MODULE_NAME};
use zircon_scene::DefaultLevelManager;

use crate::layout::{MainHostPageLayout, MainPageId, WorkbenchLayout};
use crate::module::module_descriptor;
use crate::project::{EditorProjectDocument, ProjectEditorWorkspace};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::{
    module, EditorManager, EditorSessionMode, NewProjectDraft, NewProjectTemplate,
    RecentProjectValidation, EDITOR_MANAGER_NAME,
};

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &std::path::Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_manager::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime
        .activate_module(MANAGER_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn empty_layout_with_page(page_id: &str) -> WorkbenchLayout {
    let page_id = MainPageId::new(page_id);
    WorkbenchLayout {
        active_main_page: page_id.clone(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: page_id,
            title: "Workbench".to_string(),
            document_workspace: crate::DocumentNode::Tabs(crate::TabStackLayout {
                tabs: Vec::new(),
                active_tab: None,
            }),
        }],
        drawers: crate::ActivityDrawerSlot::ALL
            .into_iter()
            .map(|slot| {
                (
                    slot,
                    crate::ActivityDrawerLayout {
                        slot,
                        tab_stack: crate::TabStackLayout::default(),
                        active_view: None,
                        mode: crate::ActivityDrawerMode::Pinned,
                        extent: if matches!(
                            slot,
                            crate::ActivityDrawerSlot::BottomLeft
                                | crate::ActivityDrawerSlot::BottomRight
                        ) {
                            200.0
                        } else {
                            260.0
                        },
                        visible: true,
                    },
                )
            })
            .collect(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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
        .get(&crate::ActivityDrawerSlot::LeftTop)
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
        .get(&crate::ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom_left = layout
        .drawers
        .get(&crate::ActivityDrawerSlot::BottomLeft)
        .expect("bottom left drawer");
    assert_eq!(
        bottom_left.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.console#1")]
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
    let crate::DocumentNode::Tabs(document_tabs) = workbench_page else {
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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
        .apply_layout_command(crate::LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::SavePreset {
            name: "authoring".to_string(),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::LoadPreset {
            name: "authoring".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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

    let world = DefaultLevelManager::default().create_default_level().snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();
    manager.open_project(&project_root).unwrap();

    manager
        .apply_layout_command(crate::LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
        .unwrap();

    let preset_asset = project_root
        .join("assets")
        .join("editor")
        .join("layout-presets")
        .join("rider.workbench-layout.json");
    assert!(preset_asset.exists(), "expected preset asset at {:?}", preset_asset);

    manager
        .apply_layout_command(crate::LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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
        host: ViewHost::Drawer(crate::ActivityDrawerSlot::LeftTop),
    };
    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: crate::DocumentNode::default(),
            }],
            drawers: BTreeMap::from([(
                crate::ActivityDrawerSlot::LeftTop,
                crate::ActivityDrawerLayout {
                    slot: crate::ActivityDrawerSlot::LeftTop,
                    tab_stack: crate::TabStackLayout {
                        tabs: vec![restored_instance.instance_id.clone()],
                        active_tab: Some(restored_instance.instance_id.clone()),
                    },
                    active_view: Some(restored_instance.instance_id.clone()),
                    mode: crate::ActivityDrawerMode::Pinned,
                    extent: 260.0,
                    visible: true,
                },
            )]),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        open_view_instances: vec![restored_instance.clone()],
        active_center_tab: None,
        active_drawers: vec![crate::ActivityDrawerSlot::LeftTop],
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let reopened = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();

    assert_eq!(reopened, restored_instance.instance_id);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(location);
}
