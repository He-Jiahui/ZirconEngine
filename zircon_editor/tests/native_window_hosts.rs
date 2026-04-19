use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::CoreRuntime;
use zircon_editor::{
    module_descriptor, EditorManager, MainPageId, NativeWindowHostState, ProjectEditorWorkspace,
    ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, WorkbenchLayout, EDITOR_MANAGER_NAME,
};
use zircon_runtime::core::manager::resolve_config_manager;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    &ENV_LOCK
}

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn editor_runtime_with_config_path(path: &std::path::Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_editor::EDITOR_MODULE_NAME)
        .unwrap();
    runtime
}

#[test]
fn reattaching_last_detached_view_clears_native_window_host_record() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_native_window_reattach");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.prefab"), None)
        .expect("prefab view should open");
    let detached = manager
        .detach_view_to_window(&instance_id)
        .expect("detach to window");
    assert!(detached);

    let layout = manager.current_layout();
    assert_eq!(layout.floating_windows.len(), 1);
    let window_id = layout.floating_windows[0].window_id.clone();
    assert_eq!(
        manager.native_window_hosts(),
        vec![NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [0.0, 0.0, 0.0, 0.0],
        }]
    );

    let reattached = manager
        .attach_view_to_target(
            &instance_id,
            ViewHost::Document(MainPageId::workbench(), vec![]),
        )
        .expect("reattach to workbench");
    assert!(reattached);
    assert!(manager.current_layout().floating_windows.is_empty());
    assert!(manager.native_window_hosts().is_empty());

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn applying_workspace_with_floating_window_syncs_native_window_bounds() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_native_window_restore");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = ViewInstanceId::new("editor.prefab#restored");
    let window_id = MainPageId::new("window:restored");
    let restored_instance = ViewInstance {
        instance_id: instance_id.clone(),
        descriptor_id: ViewDescriptorId::new("editor.prefab"),
        title: "Prefab Editor".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let mut layout = WorkbenchLayout::default();
    layout
        .floating_windows
        .push(zircon_editor::FloatingWindowLayout {
            window_id: window_id.clone(),
            title: "Prefab".to_string(),
            workspace: zircon_editor::DocumentNode::Tabs(zircon_editor::TabStackLayout {
                tabs: vec![instance_id.clone()],
                active_tab: Some(instance_id),
            }),
            focused_view: Some(ViewInstanceId::new("editor.prefab#restored")),
            frame: zircon_editor::ShellFrame::new(120.0, 80.0, 640.0, 480.0),
        });
    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: layout,
        open_view_instances: vec![restored_instance],
        active_center_tab: None,
        active_drawers: Vec::new(),
    };

    manager
        .apply_project_workspace(Some(workspace))
        .expect("apply floating workspace");

    assert_eq!(
        manager.native_window_hosts(),
        vec![NativeWindowHostState {
            window_id,
            handle: None,
            bounds: [120.0, 80.0, 640.0, 480.0],
        }]
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn native_window_hosts_remain_empty_after_config_bootstrap() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_native_window_config_bootstrap");
    let runtime = editor_runtime_with_config_path(&path);
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(WorkbenchLayout::default()).unwrap(),
        )
        .unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(manager.native_window_hosts().is_empty());

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
