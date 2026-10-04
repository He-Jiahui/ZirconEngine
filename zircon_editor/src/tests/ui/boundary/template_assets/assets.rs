use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::support::{
    assert_no_files_with_extension, assert_no_legacy_ui_document_suffixes, collect_zui_files,
    source,
};

#[test]
fn build_script_tracks_editor_assets_not_deleted_ui_sources() {
    let build = source("build.rs");
    let workbench_reference =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/editor/reference/workbench.png");

    assert!(build.contains("emit_rerun_if_changed_recursive(\"assets\")"));
    assert!(
        workbench_reference.exists(),
        "workbench reference baseline must live under tracked editor assets"
    );
    assert!(!build.contains("emit_rerun_if_changed_recursive(\"ui\")"));
    assert!(!build.contains("compile_retained_ui"));
}

#[test]
fn active_editor_ui_tree_contains_no_deleted_source_files() {
    assert_no_files_with_extension(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
        &["sli", "nt"].concat(),
    );
}

#[test]
fn production_ui_assets_use_only_zui_suffix() {
    let editor_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let runtime_root = workspace_root.join("zircon_runtime/assets/ui");
    let plugins_root = workspace_root.join("zircon_plugins");

    assert_no_legacy_ui_document_suffixes(editor_root);
    assert_no_legacy_ui_document_suffixes(runtime_root);
    if plugins_root.exists() {
        for entry in fs::read_dir(&plugins_root)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", plugins_root.display()))
        {
            let plugin_root = entry
                .unwrap_or_else(|error| {
                    panic!("read entry under `{}`: {error}", plugins_root.display())
                })
                .path();
            assert_no_legacy_ui_document_suffixes(plugin_root.join("editor"));
        }
    }
}

#[test]
fn production_zui_component_assets_are_single_component_documents() {
    let editor_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime/assets/ui");

    let mut asset_ids = BTreeMap::<String, PathBuf>::new();
    let mut component_count = 0usize;
    for path in collect_zui_files(&editor_root)
        .into_iter()
        .chain(collect_zui_files(&runtime_root))
    {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse .zui {:?}: {error}", path));
        if document.asset.kind != UiV2AssetKind::Component {
            continue;
        }
        component_count += 1;
        let (component_name, component) = document
            .components
            .iter()
            .next()
            .unwrap_or_else(|| panic!("{:?} should declare one component", path));
        assert!(
            document.nodes.contains_key(&component.root),
            "{:?} component `{}` root node `{}` should exist",
            path,
            component_name,
            component.root
        );
        assert!(
            asset_ids
                .insert(document.asset.id.clone(), path.clone())
                .is_none(),
            ".zui asset id `{}` should be unique",
            document.asset.id
        );
    }

    assert!(
        component_count > 0,
        "production UI asset roots should contain .zui component assets"
    );
}

#[test]
fn editor_v2_replacement_assets_do_not_keep_same_name_v1_sources() {
    for relative in [
        "assets/ui/editor/animation_editor.ui.toml",
        "assets/ui/editor/assets_activity.ui.toml",
        "assets/ui/editor/component_showcase.ui.toml",
        "assets/ui/editor/console.ui.toml",
        "assets/ui/editor/hierarchy.ui.toml",
        "assets/ui/editor/inspector.ui.toml",
        "assets/ui/editor/project_overview.ui.toml",
        "assets/ui/editor/welcome.ui.toml",
        "assets/ui/editor/host/activity_drawer_window.v2.ui.toml",
        "assets/ui/editor/host/activity_drawer_window.ui.toml",
        "assets/ui/editor/host/animation_graph_body.ui.toml",
        "assets/ui/editor/host/animation_sequence_body.ui.toml",
        "assets/ui/editor/host/asset_surface_controls.ui.toml",
        "assets/ui/editor/host/build_export_desktop_body.ui.toml",
        "assets/ui/editor/host/console_body.ui.toml",
        "assets/ui/editor/host/editor_main_frame.ui.toml",
        "assets/ui/editor/host/floating_window_source.ui.toml",
        "assets/ui/editor/host/hierarchy_body.ui.toml",
        "assets/ui/editor/host/inspector_body.ui.toml",
        "assets/ui/editor/host/inspector_surface_controls.ui.toml",
        "assets/ui/editor/host/module_plugins_body.ui.toml",
        "assets/ui/editor/host/pane_surface_controls.ui.toml",
        "assets/ui/editor/host/runtime_diagnostics_body.ui.toml",
        "assets/ui/editor/host/scene_viewport_toolbar.ui.toml",
        "assets/ui/editor/host/startup_welcome_controls.ui.toml",
        "assets/ui/editor/host/workbench_drawer_source.v2.ui.toml",
        "assets/ui/editor/host/workbench_drawer_source.ui.toml",
        "assets/ui/editor/host/workbench_shell.ui.toml",
        "assets/ui/editor/windows/asset_window.ui.toml",
        "assets/ui/editor/windows/ui_layout_editor_window.ui.toml",
        "assets/ui/editor/windows/workbench_window.ui.toml",
        "assets/ui/editor/workbench_activity_rail.ui.toml",
        "assets/ui/editor/workbench_dock_header.ui.toml",
        "assets/ui/editor/workbench_menu_chrome.ui.toml",
        "assets/ui/editor/workbench_menu_popup.ui.toml",
        "assets/ui/editor/workbench_page_chrome.ui.toml",
        "assets/ui/editor/workbench_status_bar.ui.toml",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        assert!(
            !path.exists(),
            "retired-suffix editor production asset must stay deleted: {relative}"
        );
    }
}
