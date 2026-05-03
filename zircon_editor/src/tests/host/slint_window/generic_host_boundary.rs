use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn editor_host_sources_do_not_depend_on_deleted_slint_trees() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scanned_roots = [
        manifest_dir
            .parent()
            .expect("workspace root")
            .join("Cargo.toml"),
        manifest_dir.join("build.rs"),
        manifest_dir.join("Cargo.toml"),
        manifest_dir.join("src"),
        manifest_dir.join("tests"),
    ];
    let forbidden = deleted_slint_source_forbidden_markers();
    let mut violations = Vec::new();

    for root in scanned_roots {
        collect_source_violations(&root, &forbidden, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "editor host must not depend on deleted Slint source trees or generated compatibility aliases:\n{}",
        violations.join("\n")
    );
}

fn collect_source_violations(path: &Path, forbidden: &[String], violations: &mut Vec<String>) {
    if path.is_dir() {
        let entries = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!("failed to read entry under {}: {error}", path.display())
            });
            collect_source_violations(&entry.path(), forbidden, violations);
        }
        return;
    }

    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml")
    ) {
        return;
    }

    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let normalized_source = source.replace('\\', "/");

    for forbidden in forbidden {
        if normalized_source.contains(forbidden) || source.contains(forbidden) {
            violations.push(format!("{} contains `{forbidden}`", path.display()));
        }
    }
}

fn deleted_slint_source_forbidden_markers() -> Vec<String> {
    vec![
        ["temp", "slint-migration"].join("/"),
        ["temp", "slint-migration"].join("\\"),
        ["slint::", "include_modules!()"].concat(),
        ["slint", "_build"].concat(),
        ["slint", "-build"].concat(),
        ["as ", "slint_ui"].concat(),
    ]
}

#[test]
fn editor_host_source_guard_rejects_hyphenated_generated_build_dependency() {
    let dependency_name = ["slint", "-build"].concat();
    let temp_root = std::env::temp_dir().join(format!(
        "zircon_editor_{}_guard_{}",
        dependency_name.replace('-', "_"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root)
        .unwrap_or_else(|error| panic!("create temp guard root failed: {error}"));
    fs::write(
        temp_root.join("Cargo.toml"),
        format!("{dependency_name} = {{ workspace = true }}\n"),
    )
    .unwrap_or_else(|error| panic!("write temp manifest failed: {error}"));

    let mut violations = Vec::new();
    collect_source_violations(
        &temp_root,
        &deleted_slint_source_forbidden_markers(),
        &mut violations,
    );
    let _ = fs::remove_dir_all(&temp_root);

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains(&dependency_name)),
        "source guard should reject generated Slint build dependencies"
    );
}

#[test]
fn editor_host_build_does_not_compile_or_stage_slint_sources() {
    let build_script = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/build.rs"));
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let build_dependency = ["slint", "-build"].concat();

    for forbidden in [
        "compile_slint_ui",
        "stage_migration_fence_slint_sources",
        "MIGRATION_FENCE_UI_SOURCE_ROOT",
    ] {
        assert!(
            !build_script.contains(forbidden),
            "editor build script must not preserve rejected Slint source compatibility through `{forbidden}`"
        );
    }
    assert!(
        !manifest.contains(&build_dependency),
        "editor manifest should not keep a Slint build dependency after the Rust-owned host cutover"
    );
}

#[test]
fn slint_host_module_exports_rust_owned_contracts_without_generated_modules() {
    let host_module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/mod.rs"
    ));

    assert!(
        !host_module.contains(&["slint::", "include_modules!()"].concat()),
        "slint_host must expose Rust-owned contracts instead of generated Slint modules"
    );
    assert!(
        host_module.contains("mod host_contract") && host_module.contains("pub(crate) use host_contract::*"),
        "slint_host should route the former generated DTO seam through a Rust-owned host_contract module"
    );
}

#[test]
fn rust_owned_host_contract_declares_window_globals_and_projection_data() {
    let window = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/host_contract/window.rs"
    ));
    let globals = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/host_contract/globals.rs"
    ));
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/host_contract/data/host_components.rs"
    ));
    let host_root = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/host_contract/data/host_root.rs"
    ));

    for required in [
        "pub(crate) struct UiHostWindow",
        "pub(crate) fn clone_strong(&self) -> Self",
        "pub(crate) fn global<T>(&self) -> T",
        "pub(crate) fn get_host_window_bootstrap(&self) -> HostWindowBootstrapData",
    ] {
        assert!(
            window.contains(required),
            "UiHostWindow is missing `{required}`"
        );
    }

    for required in [
        "pub(crate) struct UiHostContext",
        "pub(crate) struct PaneSurfaceHostContext",
        "on_host_drag_pointer_event",
        "on_host_resize_pointer_event",
        "on_viewport_toolbar_pointer_clicked",
        "on_component_showcase_option_selected",
        "on_asset_control_changed",
    ] {
        assert!(
            globals.contains(required),
            "host globals are missing `{required}`"
        );
    }

    for required in [
        "pub(crate) struct HostWindowBootstrapData",
        "pub(crate) struct HostWindowSurfaceMetricsData",
        "pub(crate) struct HostWindowSurfaceOrchestrationData",
        "pub(crate) struct HostMenuChromeData",
        "pub(crate) struct HostSideDockSurfaceData",
        "pub(crate) struct HostNativeFloatingWindowSurfaceData",
    ] {
        assert!(
            host_components.contains(required),
            "host component DTOs are missing `{required}`"
        );
    }
    assert!(
        host_root.contains("pub(crate) struct HostWindowPresentationData"),
        "host root DTOs should own the presentation payload once Slint generation is removed"
    );
}

#[test]
fn rust_owned_host_window_run_uses_native_event_loop() {
    let window = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/host_contract/window.rs"
    ));

    for required in [
        "winit::application::ApplicationHandler",
        "EventLoop::new()?",
        "run_app",
        "WindowEvent::CloseRequested",
    ] {
        assert!(
            window.contains(required),
            "UiHostWindow::run should be backed by a native event loop and is missing `{required}`"
        );
    }

    assert!(
        !window.contains("pub(crate) fn run(&self) -> Result<(), PlatformError> {\n        self.show()\n    }"),
        "UiHostWindow::run must not immediately return after marking the contract visible"
    );
}

#[test]
fn editor_ui_toml_assets_are_the_host_chrome_authority() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (relative, markers) in [
        (
            "assets/ui/editor/workbench_menu_chrome.ui.toml",
            &["WorkbenchMenuBarRoot", "MenuSlot0", "MenuSlot5"][..],
        ),
        (
            "assets/ui/editor/workbench_activity_rail.ui.toml",
            &[
                "ActivityRailPanel",
                "ActivityRailButton0",
                "ActivityRailButton1",
            ][..],
        ),
        (
            "assets/ui/editor/workbench_status_bar.ui.toml",
            &["WorkbenchStatusBar", "StatusPrimary", "ViewportLabel"][..],
        ),
        (
            "assets/ui/editor/host/scene_viewport_toolbar.ui.toml",
            &["SceneViewportToolbarRoot", "SetTool", "FrameSelection"][..],
        ),
    ] {
        let path = manifest_dir.join(relative);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for marker in markers {
            assert!(source.contains(marker), "{relative} is missing `{marker}`");
        }
    }
}

#[test]
fn generic_host_catalog_uses_shared_runtime_component_names() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let legacy_icon_button = ["UiHost", "IconButton"].concat();
    let legacy_label = ["UiHost", "Label"].concat();
    let forbidden = [legacy_icon_button, legacy_label];
    let scanned_roots = [
        manifest_dir.join("assets/ui/editor/host"),
        manifest_dir.join("src/ui/template_runtime/slint_adapter.rs"),
        manifest_dir.join("src/tests/host/template_runtime"),
        manifest_dir.join("src/tests/ui/template"),
    ];
    let mut violations = Vec::new();

    for root in scanned_roots {
        collect_source_violations(&root, &forbidden, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "generic host catalog must use shared Runtime UI `IconButton`/`Label` component names instead of host-specific transitional names:\n{}",
        violations.join("\n")
    );
}

#[test]
fn builtin_host_runtime_exposes_only_generic_host_window_document_id() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let legacy_document_id = ["workbench", "shell"].join(".");
    let legacy_constant = ["LEGACY_HOST_WINDOW", "DOCUMENT_ID"].join("_");
    let forbidden = [legacy_document_id, legacy_constant];
    let scanned_roots = [
        manifest_dir.join("src/ui/template_runtime"),
        manifest_dir.join("src/tests/host/template_runtime"),
        manifest_dir.join("src/tests/ui/template"),
    ];
    let mut violations = Vec::new();

    for root in scanned_roots {
        collect_source_violations(&root, &forbidden, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "builtin host template registration must expose the generic `ui.host_window` document id without preserving the legacy workbench shell alias:\n{}",
        violations.join("\n")
    );
}
