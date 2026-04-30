use std::path::{Path, PathBuf};

fn editor_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn assert_no_slint_sources_under(path: PathBuf) {
    if !path.exists() {
        return;
    }

    let entries = std::fs::read_dir(&path)
        .unwrap_or_else(|error| panic!("read directory `{}`: {error}", path.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("read entry in `{}`: {error}", path.display()));
        let entry_path = entry.path();
        if entry_path.is_dir() {
            assert_no_slint_sources_under(entry_path);
            continue;
        }
        assert_ne!(
            entry_path
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("slint"),
            "active Slint source should remain absent from `{}`",
            entry_path.display()
        );
    }
}

#[test]
fn active_editor_ui_tree_contains_no_slint_sources() {
    let root = editor_root();
    for absent in ["ui/workbench.slint", "ui/workbench/host_context.slint"] {
        assert!(
            !root.join(absent).exists(),
            "active Slint source `{absent}` should not exist under zircon_editor/ui"
        );
    }

    assert_no_slint_sources_under(root.join("ui"));
}

#[test]
fn editor_ui_toml_assets_replace_former_workbench_source_roles() {
    let root = editor_root();
    for required in [
        "assets/ui/editor/host/workbench_shell.ui.toml",
        "assets/ui/editor/workbench_menu_chrome.ui.toml",
        "assets/ui/editor/workbench_page_chrome.ui.toml",
        "assets/ui/editor/workbench_dock_header.ui.toml",
        "assets/ui/editor/workbench_activity_rail.ui.toml",
        "assets/ui/editor/workbench_status_bar.ui.toml",
        "assets/ui/editor/welcome.ui.toml",
        "assets/ui/editor/ui_asset_editor.ui.toml",
        "assets/ui/editor/component_showcase.ui.toml",
    ] {
        assert!(
            root.join(required).exists(),
            "missing Runtime UI asset `{required}`"
        );
    }
}

#[test]
fn rust_projection_module_remains_the_editor_host_layout_authority() {
    let root = editor_root();
    assert!(
        root.join("src/ui/layouts/windows/workbench_host_window/mod.rs")
            .exists(),
        "current Rust host projection seam should remain available during DTO cutover"
    );
    let windows_mod = std::fs::read_to_string(root.join("src/ui/layouts/windows/mod.rs"))
        .expect("windows mod should be readable");
    assert!(
        windows_mod.contains("pub(crate) mod workbench_host_window;"),
        "windows module should expose the Rust projection seam"
    );
}
