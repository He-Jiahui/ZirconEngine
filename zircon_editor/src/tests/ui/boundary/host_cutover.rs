#[test]
fn editor_manager_becomes_thin_facade_over_editor_ui_host() {
    let host_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host");
    let host_owner = host_root.join("editor_ui_host.rs");
    assert!(
        host_owner.exists(),
        "expected unified ui host owner file under {:?}",
        host_root
    );

    let host_mod = std::fs::read_to_string(host_root.join("mod.rs")).expect("ui host mod");
    assert!(
        host_mod.contains("mod editor_ui_host;"),
        "expected ui::host mod wiring to include editor_ui_host"
    );

    let host_source = std::fs::read_to_string(&host_owner).expect("editor ui host");
    for required in [
        "pub(super) core: CoreHandle",
        "pub(super) view_registry: Mutex<ViewRegistry>",
        "pub(super) layout_manager: LayoutManager",
        "pub(super) window_host_manager: Mutex<WindowHostManager>",
        "pub(super) session: Mutex<EditorSessionState>",
        "pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>",
    ] {
        assert!(
            host_source.contains(required),
            "expected editor_ui_host.rs to own `{required}`"
        );
    }

    let manager_source =
        std::fs::read_to_string(host_root.join("editor_manager.rs")).expect("editor manager");
    assert!(
        manager_source.contains("host: EditorUiHost"),
        "expected EditorManager to delegate to EditorUiHost"
    );
    for forbidden in [
        "pub(super) core: CoreHandle",
        "pub(super) view_registry: Mutex<ViewRegistry>",
        "pub(super) layout_manager: LayoutManager",
        "pub(super) window_host_manager: Mutex<WindowHostManager>",
        "pub(super) session: Mutex<EditorSessionState>",
        "pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>",
    ] {
        assert!(
            !manager_source.contains(forbidden),
            "expected EditorManager to stop directly owning `{forbidden}`"
        );
    }
}

#[test]
fn editor_ui_host_owns_bootstrap_orchestration() {
    let host_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host");
    let host_source = std::fs::read_to_string(host_root.join("editor_ui_host.rs"))
        .expect("editor ui host");
    let manager_source =
        std::fs::read_to_string(host_root.join("editor_manager.rs")).expect("editor manager");

    for required in [
        "pub(super) fn bootstrap(core: CoreHandle) -> Result<Self, EditorError>",
        "host.register_builtin_views()?",
        "host.bootstrap_default_layout()?",
    ] {
        assert!(
            host_source.contains(required),
            "expected EditorUiHost bootstrap ownership for `{required}`"
        );
    }

    for forbidden in [
        "host.register_builtin_views().expect(\"builtin editor views\")",
        "host.bootstrap_default_layout().expect(\"default workbench\")",
    ] {
        assert!(
            !manager_source.contains(forbidden),
            "expected EditorManager constructor to stop directly owning `{forbidden}`"
        );
    }
    assert!(
        manager_source.contains("EditorUiHost::bootstrap(core)"),
        "expected EditorManager to delegate bootstrap to EditorUiHost"
    );
}

#[test]
fn editor_asset_host_services_move_under_ui_host() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_host_root = crate_root.join("ui").join("host");
    let core_host_root = crate_root.join("core").join("host");
    let core_mod_source =
        std::fs::read_to_string(crate_root.join("core").join("mod.rs")).expect("core mod");

    assert!(
        ui_host_root
            .join("editor_asset_manager")
            .join("mod.rs")
            .exists(),
        "expected editor asset manager subtree under {:?}",
        ui_host_root
    );
    assert!(
        ui_host_root.join("resource_access.rs").exists(),
        "expected host resource access helper under {:?}",
        ui_host_root
    );

    let ui_host_mod = std::fs::read_to_string(ui_host_root.join("mod.rs")).expect("ui host mod");
    assert!(
        ui_host_mod.contains("pub(crate) mod editor_asset_manager;"),
        "expected ui::host mod wiring to include editor_asset_manager"
    );
    assert!(
        ui_host_mod.contains("pub(crate) mod resource_access;"),
        "expected ui::host mod wiring to include resource_access"
    );
    assert!(
        !core_mod_source.contains("pub mod host;"),
        "expected core root to stop wiring `pub mod host;` after ui host cutover"
    );
    assert!(
        !core_host_root.exists(),
        "expected core/host subtree to be deleted after ui host cutover"
    );
}

#[test]
fn editor_module_owner_moves_under_ui_host() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let ui_host_root = crate_root.join("ui").join("host");
    let ui_host_mod = std::fs::read_to_string(ui_host_root.join("mod.rs")).expect("ui host mod");
    let lib_source = std::fs::read_to_string(crate_root.join("lib.rs")).expect("editor lib");

    assert!(
        ui_host_root.join("module.rs").exists(),
        "expected EditorModule owner file under {:?}",
        ui_host_root
    );
    assert!(
        ui_host_mod.contains("pub(crate) mod module;"),
        "expected ui::host mod wiring to include module owner"
    );
    assert!(
        lib_source.contains("pub use ui::host::module::{"),
        "expected crate root to re-export EditorModule from ui::host::module"
    );
    assert!(
        !lib_source.contains("pub use core::host::module::{"),
        "expected crate root to stop re-exporting EditorModule from core::host::module"
    );
}

#[test]
fn editor_ui_host_owns_layout_project_and_workspace_orchestration() {
    let host_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host");
    let host_mod = std::fs::read_to_string(host_root.join("mod.rs")).expect("ui host mod");

    for required in [
        "mod editor_manager_layout;",
        "mod editor_manager_project;",
        "mod editor_manager_workspace;",
    ] {
        assert!(
            host_mod.contains(required),
            "expected ui::host mod wiring to include `{required}`"
        );
    }

    for owner in [
        "view_registry.rs",
        "project_access.rs",
        "layout_persistence.rs",
        "layout_commands.rs",
        "workspace_state.rs",
    ] {
        let owner_source =
            std::fs::read_to_string(host_root.join(owner)).unwrap_or_else(|_| panic!("{owner}"));
        assert!(
            owner_source.contains("impl EditorUiHost"),
            "expected {owner} ownership to move under EditorUiHost"
        );
        assert!(
            !owner_source.contains("impl EditorManager"),
            "expected {owner} to stop implementing EditorManager directly"
        );
    }

    for wrapper in [
        "editor_manager_layout.rs",
        "editor_manager_project.rs",
        "editor_manager_workspace.rs",
    ] {
        assert!(
            host_root.join(wrapper).exists(),
            "expected thin EditorManager wrapper file {wrapper} under {:?}",
            host_root
        );
    }

    let layout_wrapper = std::fs::read_to_string(host_root.join("editor_manager_layout.rs"))
        .expect("editor manager layout wrapper");
    for required in [
        "self.host.apply_layout_command(cmd)",
        "self.host.open_view(descriptor_id, target_host)",
        "self.host.close_view(instance_id)",
        "self.host.focus_view(instance_id)",
        "self.host.detach_view_to_window(instance_id)",
        "self.host.attach_view_to_target(instance_id, drop_target)",
        "self.host.save_global_default_layout()",
        "self.host.preset_names()",
    ] {
        assert!(
            layout_wrapper.contains(required),
            "expected editor_manager_layout.rs to stay as a thin wrapper for `{required}`"
        );
    }
    for forbidden in ["fn attach_instance("] {
        assert!(
            !layout_wrapper.contains(forbidden),
            "expected editor_manager_layout.rs to drop unused manager helper `{forbidden}`"
        );
    }

    let project_wrapper = std::fs::read_to_string(host_root.join("editor_manager_project.rs"))
        .expect("editor manager project wrapper");
    for required in [
        "self.host.open_project(path)",
        "self.host.save_project(path, world)",
        "self.host.create_runtime_level(scene)",
    ] {
        assert!(
            project_wrapper.contains(required),
            "expected editor_manager_project.rs to stay as a thin wrapper for `{required}`"
        );
    }
    for forbidden in [
        "fn config_manager(",
        "fn asset_manager(",
        "fn editor_asset_manager(",
        "fn current_project_root(",
        "fn resolve_ui_asset_path(",
    ] {
        assert!(
            !project_wrapper.contains(forbidden),
            "expected editor_manager_project.rs to drop internal helper `{forbidden}`"
        );
    }

    let workspace_wrapper = std::fs::read_to_string(host_root.join("editor_manager_workspace.rs"))
        .expect("editor manager workspace wrapper");
    for required in [
        "self.host.current_layout()",
        "self.host.current_view_instances()",
        "update_view_instance_metadata(instance_id, title, dirty, payload)",
        "self.host.native_window_hosts()",
        "sync_native_window_projection_bounds(window_id, bounds)",
        "self.host.descriptors()",
        "self.host.restore_workspace(policy)",
        "self.host.project_workspace()",
        "self.host.apply_project_workspace(workspace)",
    ] {
        assert!(
            workspace_wrapper.contains(required),
            "expected editor_manager_workspace.rs to stay as a thin wrapper for `{required}`"
        );
    }
    for forbidden in [
        "self.restore_ui_asset_editor_instance(&instance)?",
        "self.host.apply_project_workspace_state(workspace)?",
        "fn bootstrap_default_layout(",
    ] {
        assert!(
            !workspace_wrapper.contains(forbidden),
            "expected editor_manager_workspace.rs to stop owning `{forbidden}`"
        );
    }
}

#[test]
fn editor_ui_host_owns_asset_and_animation_editor_sessions() {
    fn collect_rs_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            for entry in std::fs::read_dir(&path).unwrap_or_else(|_| panic!("{path:?}")) {
                let entry = entry.expect("dir entry");
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
    }

    let host_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host");
    let host_mod = std::fs::read_to_string(host_root.join("mod.rs")).expect("ui host mod");

    for required in [
        "mod editor_manager_asset_editor;",
        "mod editor_manager_animation_editor;",
    ] {
        assert!(
            host_mod.contains(required),
            "expected ui::host mod wiring to include `{required}`"
        );
    }

    for subtree in ["asset_editor_sessions", "animation_editor_sessions"] {
        let mut saw_ui_host_owner = false;
        for file in collect_rs_files(&host_root.join(subtree)) {
            let source = std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("{file:?}"));
            assert!(
                !source.contains("impl EditorManager"),
                "expected {file:?} to stop implementing EditorManager directly"
            );
            saw_ui_host_owner |= source.contains("impl EditorUiHost");
        }
        assert!(
            saw_ui_host_owner,
            "expected {subtree} subtree to move ownership under EditorUiHost"
        );
    }

    let asset_wrapper = std::fs::read_to_string(host_root.join("editor_manager_asset_editor.rs"))
        .expect("asset editor wrapper");
    for required in [
        "self.host.open_ui_asset_editor(path, mode)",
        "self.host.open_ui_asset_editor_by_id(asset_id, mode)",
        "self.host.ui_asset_editor_reflection(instance_id)",
        "self.host.save_ui_asset_editor(instance_id)",
    ] {
        assert!(
            asset_wrapper.contains(required),
            "expected asset wrapper to stay thin for `{required}`"
        );
    }
    assert!(
        !asset_wrapper.contains("fn restore_ui_asset_editor_instance("),
        "expected asset wrapper to drop unused restore_ui_asset_editor_instance helper"
    );

    let animation_wrapper =
        std::fs::read_to_string(host_root.join("editor_manager_animation_editor.rs"))
            .expect("animation editor wrapper");
    for required in [
        "self.host.animation_editor_pane_presentation(instance_id)",
        "self.host.apply_animation_event(event)",
    ] {
        assert!(
            animation_wrapper.contains(required),
            "expected animation wrapper to stay thin for `{required}`"
        );
    }
}

#[test]
fn editor_ui_host_owns_startup_and_welcome_orchestration() {
    fn collect_rs_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            for entry in std::fs::read_dir(&path).unwrap_or_else(|_| panic!("{path:?}")) {
                let entry = entry.expect("dir entry");
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
    }

    let host_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host");
    let host_mod = std::fs::read_to_string(host_root.join("mod.rs")).expect("ui host mod");

    assert!(
        host_mod.contains("mod editor_manager_startup;"),
        "expected ui::host mod wiring to include startup wrapper"
    );

    let startup_root = host_root.join("startup");
    let mut saw_ui_host_owner = false;
    for file in collect_rs_files(&startup_root) {
        let source = std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("{file:?}"));
        if file.ends_with("canonical_project_root.rs") || file.ends_with("welcome_view.rs") {
            continue;
        }
        assert!(
            !source.contains("impl EditorManager"),
            "expected {file:?} to stop implementing EditorManager directly"
        );
        saw_ui_host_owner |= source.contains("impl EditorUiHost");
    }
    assert!(
        saw_ui_host_owner,
        "expected startup subtree to move ownership under EditorUiHost"
    );

    let startup_wrapper = std::fs::read_to_string(host_root.join("editor_manager_startup.rs"))
        .expect("startup wrapper");
    for required in [
        "self.host.resolve_startup_session()",
        "self.host.open_project_and_remember(path)",
        "self.host.create_project_and_open(draft)",
        "self.host.recent_projects_snapshot()",
        "self.host.forget_recent_project(path)",
        "self.host.update_recent_project(path, display_name)",
        "self.host.show_welcome_page()",
        "self.host.dismiss_welcome_page()",
    ] {
        assert!(
            startup_wrapper.contains(required),
            "expected startup wrapper to stay thin for `{required}`"
        );
    }
}
